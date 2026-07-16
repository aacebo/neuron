use rust_bert::pipelines::token_classification::{Token, TokenClassificationModel};

use crate::{CortexError, CortexInput, CortexOutput, Routine, types};

pub struct PIIExtraction<'a> {
    model: &'a TokenClassificationModel,
}

impl<'a> PIIExtraction<'a> {
    pub fn new(model: &'a TokenClassificationModel) -> Self {
        Self { model }
    }
}

impl<'a> Routine for PIIExtraction<'a> {
    fn name(&self) -> &'static str {
        "pii-extraction"
    }

    fn invoke(&self, input: CortexInput<'_>) -> Result<CortexOutput, CortexError> {
        let out = self.model.predict(input.text, true, false);
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
    finished: Vec<PiiEntity>,
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

    fn push(&mut self, token: Token) {
        let Some(offset) = token.offset else {
            self.finish_current();
            return;
        };

        let Some((tag, label)) = EntityTag::parse_label(&token.label) else {
            self.finish_current();
            return;
        };

        if self.should_start_new(tag, label, offset.begin) {
            self.finish_current();
            self.current = Some(PiiEntityBuilder::new(&token, tag, label, offset.begin, offset.end));
        } else if let Some(current) = self.current.as_mut() {
            current.push(&token, tag, offset.end);
        }

        if tag.is_terminal() {
            self.finish_current();
        }
    }

    fn finish(mut self) -> Vec<types::CortexAnnotation> {
        self.finish_current();
        self.finished.into_iter().map(|entity| entity.annotation).collect()
    }

    fn should_start_new(&self, tag: EntityTag, label: &str, start: u32) -> bool {
        match self.current.as_ref() {
            Some(current) => {
                if current.is_closed() || !current.has_label(label) {
                    return true;
                }

                tag.is_start() && !current.is_contiguous_with(start)
            }
            None => true,
        }
    }

    fn finish_current(&mut self) {
        if let Some(builder) = self.current.take() {
            let annotation = builder.finish();

            if annotation.annotation.score >= self.min_score {
                self.push_finished(annotation);
            }
        }
    }

    fn push_finished(&mut self, entity: PiiEntity) {
        if let Some(previous) = self.finished.last_mut()
            && previous.can_merge(&entity)
        {
            previous.merge(entity);
            return;
        }

        self.finished.push(entity);
    }
}

struct PiiEntity {
    annotation: types::CortexAnnotation,
    token_count: usize,
}

impl PiiEntity {
    fn can_merge(&self, other: &Self) -> bool {
        self.annotation.label.eq_ignore_ascii_case(&other.annotation.label)
            && self.annotation.spans.first().map(|span| span.end) == other.annotation.spans.first().map(|span| span.start)
    }

    fn merge(&mut self, other: Self) {
        let total_tokens = self.token_count + other.token_count;

        self.annotation.text = format!("{} {}", self.annotation.text, other.annotation.text);
        self.annotation.score = ((self.annotation.score * self.token_count as f64)
            + (other.annotation.score * other.token_count as f64))
            / total_tokens as f64;
        self.token_count = total_tokens;

        if let (Some(current), Some(next)) = (self.annotation.spans.first_mut(), other.annotation.spans.first()) {
            current.end = next.end;
        }
    }
}

struct PiiEntityBuilder {
    label: String,
    words: Vec<String>,
    score_sum: f64,
    token_count: usize,
    start: u32,
    end: u32,
    previous_tag: EntityTag,
}

impl PiiEntityBuilder {
    fn new(token: &Token, tag: EntityTag, label: &str, start: u32, end: u32) -> Self {
        Self {
            label: label.to_lowercase(),
            words: vec![token.text.clone()],
            score_sum: token.score,
            token_count: 1,
            start,
            end,
            previous_tag: tag,
        }
    }

    fn has_label(&self, label: &str) -> bool {
        self.label.eq_ignore_ascii_case(label)
    }

    fn is_closed(&self) -> bool {
        self.previous_tag.is_terminal()
    }

    fn is_contiguous_with(&self, start: u32) -> bool {
        self.end == start
    }

    fn push(&mut self, token: &Token, tag: EntityTag, end: u32) {
        self.words.push(token.text.clone());
        self.score_sum += token.score;
        self.token_count += 1;
        self.end = end;
        self.previous_tag = tag;
    }

    fn finish(self) -> PiiEntity {
        PiiEntity {
            annotation: types::CortexAnnotation {
                r#type: String::from("pii"),
                label: self.label,
                text: self.words.join(" "),
                score: self.score_sum / self.token_count as f64,
                spans: vec![types::Span::new(self.start, self.end)],
            },
            token_count: self.token_count,
        }
    }
}
