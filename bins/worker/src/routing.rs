#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RoutingPolicy {
    pub candidate_limit: u32,
    pub min_confidence: f64,
    pub ambiguity_margin: f64,
}

impl RoutingPolicy {
    const SCORE_EPSILON: f64 = 1e-12;

    pub fn new(candidate_limit: u32, min_confidence: f64, ambiguity_margin: f64) -> error::Result<Self> {
        let policy = Self {
            candidate_limit,
            min_confidence,
            ambiguity_margin,
        };

        policy.validate()?;
        Ok(policy)
    }

    pub fn validate(&self) -> error::Result<()> {
        if self.candidate_limit < 2 {
            return Err(error::config("routing candidate limit must be at least 2"));
        }

        if !self.min_confidence.is_finite() || !(-1.0..=1.0).contains(&self.min_confidence) {
            return Err(error::config(
                "routing minimum confidence must be finite and between -1 and 1",
            ));
        }

        if !self.ambiguity_margin.is_finite() || !(0.0..=2.0).contains(&self.ambiguity_margin) {
            return Err(error::config("routing ambiguity margin must be finite and between 0 and 2"));
        }

        Ok(())
    }

    pub fn decide<T: Clone>(&self, mut candidates: Vec<storage::SearchResult<T>>) -> RoutingDecision<T> {
        candidates.retain(|candidate| candidate.similarity.is_finite());
        candidates.sort_by(|a, b| b.similarity.total_cmp(&a.similarity));
        candidates.truncate(self.candidate_limit as usize);

        let Some(top) = candidates.first() else {
            return RoutingDecision::NoRoute {
                reason: NoRouteReason::NoCandidates,
                candidates,
            };
        };

        if top.similarity < self.min_confidence {
            return RoutingDecision::NoRoute {
                reason: NoRouteReason::LowConfidence,
                candidates,
            };
        }

        let top_similarity = top.similarity;
        let mut agents = vec![top.clone()];
        agents.extend(
            candidates
                .iter()
                .skip(1)
                .filter(|candidate| {
                    candidate.similarity >= self.min_confidence
                        && top_similarity - candidate.similarity + Self::SCORE_EPSILON < self.ambiguity_margin
                })
                .cloned(),
        );

        RoutingDecision::Selected { agents, candidates }
    }
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        Self {
            candidate_limit: 5,
            min_confidence: 0.20,
            ambiguity_margin: 0.05,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoRouteReason {
    NoCandidates,
    LowConfidence,
}

impl NoRouteReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoCandidates => "no_candidates",
            Self::LowConfidence => "low_confidence",
        }
    }
}

#[derive(Debug, Clone)]
pub enum RoutingDecision<T> {
    Selected {
        agents: Vec<storage::SearchResult<T>>,
        candidates: Vec<storage::SearchResult<T>>,
    },
    NoRoute {
        reason: NoRouteReason,
        candidates: Vec<storage::SearchResult<T>>,
    },
}

#[cfg(test)]
mod tests {
    use storage::SearchResult;

    use super::{NoRouteReason, RoutingDecision, RoutingPolicy};

    fn candidate(name: &'static str, similarity: f64) -> SearchResult<&'static str> {
        SearchResult {
            entity: name,
            similarity,
        }
    }

    fn selected(decision: RoutingDecision<&'static str>) -> Vec<&'static str> {
        match decision {
            RoutingDecision::Selected { agents, .. } => agents.into_iter().map(|agent| agent.entity).collect(),
            RoutingDecision::NoRoute { reason, .. } => panic!("expected selection, got {reason:?}"),
        }
    }

    #[test]
    fn selects_clear_code_review_winner() {
        let policy = RoutingPolicy::default();
        let decision = policy.decide(vec![
            candidate("calendar_agent", 0.129712),
            candidate("code_review_agent", 0.267148),
        ]);
        assert_eq!(selected(decision), ["code_review_agent"]);
    }

    #[test]
    fn selects_all_credible_ambiguous_agents() {
        let policy = RoutingPolicy::default();
        let decision = policy.decide(vec![
            candidate("code_review_agent", 0.442543),
            candidate("expense_agent", 0.458708),
            candidate("calendar_agent", 0.19),
        ]);
        assert_eq!(selected(decision), ["expense_agent", "code_review_agent"]);
    }

    #[test]
    fn no_routes_low_confidence_candidates() {
        let decision = RoutingPolicy::default().decide(vec![candidate("calendar_agent", 0.086917)]);
        assert!(matches!(
            decision,
            RoutingDecision::NoRoute {
                reason: NoRouteReason::LowConfidence,
                ..
            }
        ));
    }

    #[test]
    fn no_routes_an_empty_candidate_set() {
        let decision = RoutingPolicy::default().decide(Vec::<SearchResult<&str>>::new());
        assert!(matches!(
            decision,
            RoutingDecision::NoRoute {
                reason: NoRouteReason::NoCandidates,
                ..
            }
        ));
    }

    #[test]
    fn selects_a_single_qualifying_candidate() {
        let decision = RoutingPolicy::default().decide(vec![candidate("code_review_agent", 0.21)]);
        assert_eq!(selected(decision), ["code_review_agent"]);
    }

    #[test]
    fn excludes_candidates_exactly_at_the_margin() {
        let decision = RoutingPolicy::default().decide(vec![candidate("first", 0.30), candidate("second", 0.25)]);
        assert_eq!(selected(decision), ["first"]);
    }

    #[test]
    fn includes_tied_candidates() {
        let decision = RoutingPolicy::default().decide(vec![candidate("first", 0.30), candidate("second", 0.30)]);
        assert_eq!(selected(decision), ["first", "second"]);
    }

    #[test]
    fn limits_the_ranked_candidate_set() {
        let policy = RoutingPolicy::new(2, 0.20, 0.10).unwrap();
        let decision = policy.decide(vec![
            candidate("third", 0.45),
            candidate("first", 0.50),
            candidate("second", 0.46),
        ]);
        assert_eq!(selected(decision), ["first", "second"]);
    }

    #[test]
    fn validates_policy_values() {
        assert!(RoutingPolicy::new(5, 0.20, 0.05).is_ok());
        assert!(RoutingPolicy::new(1, 0.20, 0.05).is_err());
        assert!(RoutingPolicy::new(5, 1.01, 0.05).is_err());
        assert!(RoutingPolicy::new(5, 0.20, -0.01).is_err());
    }

    #[test]
    #[ignore = "loads the configured embedding model and requires its model cache"]
    fn model_backed_http_fixture_calibration() {
        let texts = vec![
            [
                "Name: calendar_agent",
                "Description: Manages calendars, availability, meetings, attendees, and reminders",
                "Skills:",
                "- Name: manage_calendar",
                "  Display name: Manage Calendar",
                "- Name: check_availability",
                "  Display name: Check Availability",
                "- Name: schedule_meeting",
                "  Display name: Schedule Meeting",
            ]
            .join("\n"),
            [
                "Name: expense_agent",
                "Description: Processes receipts, categorizes expenses, and creates reimbursement reports",
                "Skills:",
                "- Name: process_receipt",
                "  Display name: Process Receipt",
                "- Name: categorize_expense",
                "  Display name: Categorize Expense",
                "- Name: create_expense_report",
                "  Display name: Create Expense Report",
            ]
            .join("\n"),
            [
                "Name: code_review_agent",
                "Description: Reviews source code and pull requests for bugs, security issues, and maintainability problems",
                "Skills:",
                "- Name: review_pull_request",
                "  Display name: Review Pull Request",
                "- Name: detect_security_issues",
                "  Display name: Detect Security Issues",
                "- Name: suggest_refactoring",
                "  Display name: Suggest Refactoring",
            ]
            .join("\n"),
            "Schedule a 30 minute meeting with Alex tomorrow afternoon and remind me 15 minutes before it starts.".into(),
            "Categorize this $42.50 restaurant receipt and add it to my reimbursement report.".into(),
            "Review pull request 184 and determine whether the authentication middleware allows requests with expired tokens to bypass authorization.".into(),
            "Explain why ocean tides change throughout the month.".into(),
            "Review a pull request that changes receipt categorization and expense reimbursement code.".into(),
        ];
        let vectors: Vec<Vec<f32>> = ai::pipelines::embeddings(ai::pipelines::TextArgs {
            text: texts,
            ..Default::default()
        })
        .unwrap()
        .into_iter()
        .map(|artifact| artifact.vector.unwrap())
        .collect();
        let agent_names = ["calendar_agent", "expense_agent", "code_review_agent"];
        let route = |query: &[f32]| {
            let candidates = agent_names
                .iter()
                .zip(&vectors[..3])
                .map(|(name, vector)| candidate(name, cosine_similarity(query, vector)))
                .collect();
            RoutingPolicy::default().decide(candidates)
        };

        assert_eq!(selected(route(&vectors[3])), ["calendar_agent"]);
        assert_eq!(selected(route(&vectors[4])), ["expense_agent"]);
        assert_eq!(selected(route(&vectors[5])), ["code_review_agent"]);
        assert!(matches!(
            route(&vectors[6]),
            RoutingDecision::NoRoute {
                reason: NoRouteReason::LowConfidence,
                ..
            }
        ));
        assert_eq!(selected(route(&vectors[7])), ["expense_agent", "code_review_agent"]);
    }

    fn cosine_similarity(left: &[f32], right: &[f32]) -> f64 {
        let dot: f64 = left
            .iter()
            .zip(right)
            .map(|(left, right)| f64::from(*left) * f64::from(*right))
            .sum();
        let left_norm = left.iter().map(|value| f64::from(*value).powi(2)).sum::<f64>().sqrt();
        let right_norm = right.iter().map(|value| f64::from(*value).powi(2)).sum::<f64>().sqrt();
        dot / (left_norm * right_norm)
    }
}
