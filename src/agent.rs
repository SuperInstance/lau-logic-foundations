//! Agent reasoning: automated theorem proving for agent behavioral contracts

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::propositional::{Formula, Var, atom, and, or, not, implies};
use crate::cnf::{Cnf, Literal, TseitinEncoder};
use crate::dpll::{DpllSolver, SatResult};
use crate::resolution::{ResolutionProver, ResClause, ResLiteral};
use crate::predicate::{Term, PredFormula, Substitution};

/// An agent behavioral contract specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContract {
    /// Name of the agent
    pub agent_name: String,
    /// Preconditions: what must be true before action
    pub preconditions: Vec<Formula>,
    /// Postconditions: what must be true after action
    pub postconditions: Vec<Formula>,
    /// Invariants: what must always hold
    pub invariants: Vec<Formula>,
    /// Safety properties: things that must never happen
    pub safety_properties: Vec<Formula>,
}

impl AgentContract {
    pub fn new(name: &str) -> Self {
        Self {
            agent_name: name.to_string(),
            preconditions: vec![],
            postconditions: vec![],
            invariants: vec![],
            safety_properties: vec![],
        }
    }

    /// Add a precondition
    pub fn requires(&mut self, f: Formula) -> &mut Self {
        self.preconditions.push(f);
        self
    }

    /// Add a postcondition
    pub fn ensures(&mut self, f: Formula) -> &mut Self {
        self.postconditions.push(f);
        self
    }

    /// Add an invariant
    pub fn invariant(&mut self, f: Formula) -> &mut Self {
        self.invariants.push(f);
        self
    }

    /// Add a safety property
    pub fn safety(&mut self, f: Formula) -> &mut Self {
        self.safety_properties.push(f);
        self
    }

    /// Check if the contract is internally consistent (preconditions don't contradict invariants)
    pub fn check_consistency(&self) -> ConsistencyResult {
        let solver = DpllSolver::new();

        // Check that preconditions are satisfiable
        for pre in &self.preconditions {
            let cnf = TseitinEncoder::tseitin_encode(pre);
            match solver.solve(&cnf) {
                SatResult::Unsatisfiable => {
                    return ConsistencyResult {
                        consistent: false,
                        issue: format!("Precondition is unsatisfiable: {:?}", pre),
                    };
                }
                _ => {}
            }
        }

        // Check that invariants are jointly satisfiable
        if !self.invariants.is_empty() {
            let mut combined = Formula::Top;
            for inv in &self.invariants {
                combined = and(combined, inv.clone());
            }
            let cnf = TseitinEncoder::tseitin_encode(&combined);
            match solver.solve(&cnf) {
                SatResult::Unsatisfiable => {
                    return ConsistencyResult {
                        consistent: false,
                        issue: "Invariants are jointly unsatisfiable".to_string(),
                    };
                }
                _ => {}
            }
        }

        ConsistencyResult {
            consistent: true,
            issue: String::new(),
        }
    }

    /// Check if a safety property holds given the contract.
    /// A safety property P holds if the invariants + preconditions entail P.
    pub fn verify_safety(&self) -> Vec<SafetyResult> {
        let mut results = Vec::new();

        for safety_prop in &self.safety_properties {
            // Check: (invariants ∧ preconditions) → safety_property
            // i.e., ¬(invariants ∧ preconditions ∧ ¬safety_property) is a tautology
            let mut lhs = Formula::Top;
            for inv in &self.invariants {
                lhs = and(lhs, inv.clone());
            }
            for pre in &self.preconditions {
                lhs = and(lhs, pre.clone());
            }
            let negated = and(lhs, not(safety_prop.clone()));
            let cnf = TseitinEncoder::tseitin_encode(&negated);
            let solver = DpllSolver::new();

            match solver.solve(&cnf) {
                SatResult::Satisfiable(_) => {
                    results.push(SafetyResult {
                        property: format!("{:?}", safety_prop),
                        holds: false,
                        counterexample: Some("Found a model violating safety".to_string()),
                    });
                }
                SatResult::Unsatisfiable => {
                    results.push(SafetyResult {
                        property: format!("{:?}", safety_prop),
                        holds: true,
                        counterexample: None,
                    });
                }
            }
        }

        results
    }
}

/// Result of a consistency check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyResult {
    pub consistent: bool,
    pub issue: String,
}

/// Result of a safety verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyResult {
    pub property: String,
    pub holds: bool,
    pub counterexample: Option<String>,
}

/// Agent knowledge base for logical reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentKnowledgeBase {
    /// Known facts (propositions believed true)
    pub facts: Vec<Formula>,
    /// Rules (implications believed true)
    pub rules: Vec<Formula>,
}

impl AgentKnowledgeBase {
    pub fn new() -> Self {
        Self { facts: vec![], rules: vec![] }
    }

    /// Add a fact
    pub fn add_fact(&mut self, f: Formula) -> &mut Self {
        self.facts.push(f);
        self
    }

    /// Add a rule
    pub fn add_rule(&mut self, f: Formula) -> &mut Self {
        self.rules.push(f);
        self
    }

    /// Query: does the knowledge base entail the given formula?
    pub fn query(&self, goal: &Formula) -> QueryResult {
        let solver = DpllSolver::new();

        // Build: facts ∧ rules ∧ ¬goal → check unsatisfiability
        let mut all = Formula::Top;
        for fact in &self.facts {
            all = and(all, fact.clone());
        }
        for rule in &self.rules {
            all = and(all, rule.clone());
        }
        let negated_goal = and(all, not(goal.clone()));
        let cnf = TseitinEncoder::tseitin_encode(&negated_goal);

        match solver.solve(&cnf) {
            SatResult::Unsatisfiable => QueryResult {
                proven: true,
                explanation: "Goal follows from knowledge base".to_string(),
            },
            SatResult::Satisfiable(assignment) => QueryResult {
                proven: false,
                explanation: format!("Countermodel found: {:?}", assignment),
            },
        }
    }
}

/// Result of a knowledge base query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub proven: bool,
    pub explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_consistency() {
        let mut contract = AgentContract::new("test_agent");
        contract.requires(atom("ready"));
        contract.invariant(atom("initialized"));
        let result = contract.check_consistency();
        assert!(result.consistent);
    }

    #[test]
    fn test_contract_safety_holds() {
        let mut contract = AgentContract::new("safe_agent");
        contract.invariant(atom("safe"));
        contract.safety(atom("safe"));
        let results = contract.verify_safety();
        assert_eq!(results.len(), 1);
        assert!(results[0].holds);
    }

    #[test]
    fn test_contract_safety_violated() {
        let mut contract = AgentContract::new("unsafe_agent");
        contract.safety(atom("never_happens"));
        // No invariants or preconditions, so ¬never_happens is satisfiable
        let results = contract.verify_safety();
        assert_eq!(results.len(), 1);
        assert!(!results[0].holds);
    }

    #[test]
    fn test_knowledge_base_query_provable() {
        let mut kb = AgentKnowledgeBase::new();
        kb.add_fact(atom("p"));
        kb.add_rule(implies(atom("p"), atom("q")));
        let result = kb.query(&atom("q"));
        assert!(result.proven);
    }

    #[test]
    fn test_knowledge_base_query_not_provable() {
        let mut kb = AgentKnowledgeBase::new();
        kb.add_fact(atom("p"));
        let result = kb.query(&atom("q"));
        assert!(!result.proven);
    }

    #[test]
    fn test_knowledge_base_chain() {
        let mut kb = AgentKnowledgeBase::new();
        kb.add_fact(atom("a"));
        kb.add_rule(implies(atom("a"), atom("b")));
        kb.add_rule(implies(atom("b"), atom("c")));
        let result = kb.query(&atom("c"));
        assert!(result.proven);
    }

    #[test]
    fn test_empty_knowledge_base() {
        let kb = AgentKnowledgeBase::new();
        let result = kb.query(&atom("anything"));
        assert!(!result.proven);
    }

    #[test]
    fn test_contract_with_preconditions() {
        let mut contract = AgentContract::new("guarded_agent");
        contract.requires(atom("authenticated"));
        contract.ensures(atom("response_sent"));
        contract.invariant(atom("alive"));
        let result = contract.check_consistency();
        assert!(result.consistent);
    }

    #[test]
    fn test_agent_reasoning_complex() {
        // If agent is active and task is queued, then task will be processed
        let mut kb = AgentKnowledgeBase::new();
        kb.add_fact(and(atom("active"), atom("task_queued")));
        kb.add_rule(implies(
            and(atom("active"), atom("task_queued")),
            atom("task_processed"),
        ));
        let result = kb.query(&atom("task_processed"));
        assert!(result.proven);
    }
}
