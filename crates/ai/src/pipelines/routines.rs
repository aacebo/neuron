use super::{RoutineResult, ScoredArgs, TextArgs, borrow, defaults, load};
use crate::models::Loaded;
use crate::tasks;
use crate::types::{Annotation, Artifact, ArtifactContent, Entity, Offset};

/// Each routine is the same four steps: parse args, load the model, ask it for the capability the
/// routine needs, and present the result. Asking is where the capability matrix bites -- a model
/// that cannot do the job fails here, by name, instead of part-way through inference.

pub fn embeddings(args: TextArgs) -> RoutineResult<Vec<Artifact>> {
    let model = load(&args.model.resolve(defaults::embed())?, &args.api_key)?;
    let capable = model.as_embed().ok_or_else(|| model.cannot("embed"))?;
    let out = tasks::embed(capable, &model.context(), &borrow(&args.text))?;
    let artifacts: Vec<Artifact> = out
        .into_iter()
        .zip(args.text)
        .map(|(vector, text)| Artifact {
            name: "embedding".to_string(),
            value: ArtifactContent::text(text),
            vector: Some(vector),
        })
        .collect();

    Ok(artifacts)
}

pub fn keywords(args: ScoredArgs) -> RoutineResult<Vec<Annotation>> {
    let model = load(&args.model.resolve(defaults::keywords())?, &args.api_key)?;
    let capable = model.as_embed().ok_or_else(|| model.cannot("embed"))?;
    let out = tasks::keywords(capable, &model.context(), &borrow(&args.text), TOP_N)?;
    let min_score = args.min_score as f32;
    let mut annotations: Vec<Annotation> = Vec::new();

    for keywords in out {
        for keyword in keywords.into_iter().filter(|k| k.score >= min_score) {
            annotations.push(Annotation {
                name: String::from("keyword"),
                label: keyword.text.clone(),
                text: keyword.text,
                score: keyword.score as f64,
                spans: keyword.offsets.iter().map(|o| Offset::new(o.begin, o.end)).collect(),
            });
        }
    }

    Ok(annotations)
}

pub fn sentiment(args: ScoredArgs) -> RoutineResult<Vec<Annotation>> {
    let model = load(&args.model.resolve(defaults::classify())?, &args.api_key)?;
    let capable = model.as_classify().ok_or_else(|| model.cannot("classify"))?;
    let out = tasks::sentiment(capable, &model.context(), &borrow(&args.text))?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for (index, sentiment) in out.into_iter().enumerate() {
        if sentiment.score < args.min_score {
            continue;
        }

        let source = args.text.get(index).map(String::as_str).unwrap_or_default();
        let label = sentiment.polarity.as_str();

        annotations.push(Annotation {
            name: String::from("sentiment"),
            label: label.to_string(),
            text: label.to_string(),
            score: sentiment.score,
            spans: vec![Offset::new(0, source.chars().count() as u32)],
        });
    }

    Ok(annotations)
}

pub fn entities(args: ScoredArgs) -> RoutineResult<Vec<Annotation>> {
    let (text, min_score, model) = token_args(args)?;
    let capable = model.as_token_classify().ok_or_else(|| model.cannot("token-classify"))?;
    let out = tasks::entities(capable, &model.context(), &borrow(&text))?;

    // The local CoNLL-03 checkpoint emits its own tag names; a hosted model already speaks the
    // long form. Both are normalised here so a manifest sees one vocabulary.
    Ok(annotate(out, "entity", min_score, |entity| match entity.label.as_str() {
        "ORG" => "organization".to_string(),
        "PER" => "person".to_string(),
        "LOC" => "location".to_string(),
        other => other.to_lowercase(),
    }))
}

pub fn pii(args: ScoredArgs) -> RoutineResult<Vec<Vec<Entity>>> {
    let (text, min_score, model) = token_args(args)?;
    let capable = model.as_token_classify().ok_or_else(|| model.cannot("token-classify"))?;
    let out = tasks::pii(capable, &model.context(), &borrow(&text), min_score)?;
    Ok(out)
}

pub fn summarize(args: TextArgs) -> RoutineResult<Vec<Artifact>> {
    let model = load(&args.model.resolve(defaults::generate())?, &args.api_key)?;
    let capable = model.as_generate().ok_or_else(|| model.cannot("generate"))?;
    let out = tasks::summarize(capable, &model.context(), &borrow(&args.text))?;
    let artifacts: Vec<Artifact> = out
        .into_iter()
        .map(|summary| Artifact {
            name: "summary".to_string(),
            value: ArtifactContent::text(summary),
            vector: None,
        })
        .collect();

    Ok(artifacts)
}

const TOP_N: usize = 5;

type TokenArgs = (Vec<String>, f64, std::sync::Arc<Loaded>);

fn token_args(args: ScoredArgs) -> RoutineResult<TokenArgs> {
    let model = load(&args.model.resolve(defaults::token_classify())?, &args.api_key)?;
    Ok((args.text, args.min_score, model))
}

fn annotate(out: Vec<Vec<Entity>>, name: &str, min_score: f64, label: impl Fn(&Entity) -> String) -> Vec<Annotation> {
    let mut annotations: Vec<Annotation> = Vec::new();

    for entities in out {
        for entity in entities.into_iter().filter(|e| e.score >= min_score) {
            annotations.push(Annotation {
                name: name.to_string(),
                label: label(&entity),
                text: entity.word,
                score: entity.score,
                spans: vec![Offset::new(entity.offset.begin, entity.offset.end)],
            });
        }
    }

    annotations
}
