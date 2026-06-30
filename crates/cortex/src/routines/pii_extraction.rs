use rust_bert::pipelines::ner::{self, Entity};

use crate::{CortexError, CortexInput, CortexOutput, Routine, types};

pub struct PIIExtraction<'a> {
    model: &'a ner::NERModel,
}

impl<'a> PIIExtraction<'a> {
    pub fn new(model: &'a ner::NERModel) -> Self {
        Self { model }
    }
}

impl<'a> Routine for PIIExtraction<'a> {
    fn name(&self) -> &'static str {
        "pii-extraction"
    }

    fn invoke(&self, input: CortexInput<'_>) -> Result<CortexOutput, CortexError> {
        let out = self.model.predict(input.text);
        let mut output = CortexOutput::default();

        for sequence in out {
            let mut entities = PiiEntities::new(input.min_score as f64);

            for entity in sequence {
                entities.push(entity);
            }

            output.annotations.extend(entities.finish());
        }

        Ok(output)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum EntityTag {
    Begin,
    Inside,
    End,
    Single,
}

impl EntityTag {
    fn parse_label(label: &str) -> Option<(Self, &str)> {
        let (tag, label) = match label.split_once('-') {
            Some(("B", label)) => (Self::Begin, label),
            Some(("I", label)) => (Self::Inside, label),
            Some(("E", label)) => (Self::End, label),
            Some(("S", label)) => (Self::Single, label),
            Some(("O", _)) | None if label == "O" => return None,
            Some(_) => return None,
            None => (Self::Single, label),
        };

        (!label.is_empty()).then_some((tag, label))
    }

    fn is_start(self) -> bool {
        matches!(self, Self::Begin | Self::Single)
    }

    fn is_terminal(self) -> bool {
        matches!(self, Self::End | Self::Single)
    }
}

struct PiiEntities {
    current: Option<PiiEntityBuilder>,
    finished: Vec<types::CortexAnnotation>,
    min_score: f64,
}

impl PiiEntities {
    fn new(min_score: f64) -> Self {
        Self {
            current: None,
            finished: Vec::new(),
            min_score,
        }
    }

    fn push(&mut self, entity: Entity) {
        let Some((tag, label)) = EntityTag::parse_label(&entity.label) else {
            self.finish_current();
            return;
        };

        if self.should_start_new(tag, label) {
            self.finish_current();
            self.current = Some(PiiEntityBuilder::new(&entity, tag, label));
        } else if let Some(current) = self.current.as_mut() {
            current.push(&entity, tag);
        }

        if tag.is_terminal() {
            self.finish_current();
        }
    }

    fn finish(mut self) -> Vec<types::CortexAnnotation> {
        self.finish_current();
        self.finished
    }

    fn should_start_new(&self, tag: EntityTag, label: &str) -> bool {
        match self.current.as_ref() {
            Some(current) => tag.is_start() || current.is_closed() || !current.has_label(label),
            None => true,
        }
    }

    fn finish_current(&mut self) {
        if let Some(builder) = self.current.take() {
            let annotation = builder.finish();

            if annotation.score >= self.min_score {
                self.finished.push(annotation);
            }
        }
    }
}

struct PiiEntityBuilder {
    label: String,
    words: Vec<String>,
    score_sum: f64,
    start: u32,
    end: u32,
    previous_tag: EntityTag,
}

impl PiiEntityBuilder {
    fn new(entity: &Entity, tag: EntityTag, label: &str) -> Self {
        Self {
            label: label.to_lowercase(),
            words: vec![entity.word.clone()],
            score_sum: entity.score,
            start: entity.offset.begin,
            end: entity.offset.end,
            previous_tag: tag,
        }
    }

    fn has_label(&self, label: &str) -> bool {
        self.label.eq_ignore_ascii_case(label)
    }

    fn is_closed(&self) -> bool {
        self.previous_tag.is_terminal()
    }

    fn push(&mut self, entity: &Entity, tag: EntityTag) {
        self.words.push(entity.word.clone());
        self.score_sum += entity.score;
        self.end = entity.offset.end;
        self.previous_tag = tag;
    }

    fn finish(self) -> types::CortexAnnotation {
        types::CortexAnnotation {
            r#type: String::from("pii"),
            label: self.label,
            text: self.words.join(" "),
            score: self.score_sum / self.words.len() as f64,
            spans: vec![types::Span::new(self.start, self.end)],
        }
    }
}
