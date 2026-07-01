use rust_bert::pipelines::keywords_extraction;

use crate::{Action, types};

pub struct Keywords<'a> {
    model: &'a keywords_extraction::KeywordExtractionModel<'a>,
}

impl<'a> Keywords<'a> {
    pub fn new(model: &'a keywords_extraction::KeywordExtractionModel) -> Self {
        Self { model }
    }

    fn span_from_byte_offsets(text: &str, start: u32, end: u32) -> types::Span {
        types::Span::new(
            Self::byte_offset_to_char_offset(text, start),
            Self::byte_offset_to_char_offset(text, end),
        )
    }

    fn byte_offset_to_char_offset(text: &str, byte_offset: u32) -> u32 {
        let byte_offset = byte_offset as usize;
        let byte_offset = byte_offset.min(text.len());

        text.char_indices()
            .take_while(|(index, _)| *index < byte_offset)
            .count() as u32
    }
}

impl<'a> Action for Keywords<'a> {
    fn name(&self) -> &'static str {
        "keywords::extract"
    }

    fn invoke(&self, ctx: &mut crate::Context) -> Result<(), Box<dyn std::error::Error>> {
        let out = self.model.predict(ctx.input)?;

        for (index, keywords) in out.into_iter().enumerate() {
            let text = ctx.input.get(index).copied().unwrap_or_default();

            for keyword in keywords
                .into_iter()
                .filter(|v| v.score >= ctx.options.min_score)
            {
                ctx.annotations.push(types::CortexAnnotation {
                    r#type: String::from("keyword"),
                    label: keyword.text.clone(),
                    text: keyword.text,
                    score: keyword.score as f64,
                    spans: keyword
                        .offsets
                        .iter()
                        .map(|offset| Self::span_from_byte_offsets(text, offset.begin, offset.end))
                        .collect(),
                });
            }
        }

        Ok(())
    }
}
