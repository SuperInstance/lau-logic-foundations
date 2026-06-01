# lau-logic-foundations

A Rust library for mathematical logic: propositional logic, predicate logic, automated reasoning (DPLL, resolution), natural deduction proofs, Gödel numbering, and agent behavioral contract verification.

## What This Does

`lau-logic-foundations` provides the core machinery of formal logic as composable Rust types and algorithms:

- **Propositional logic** — syntax trees, truth tables, tautology/satisfiability checking, entailment, equivalence.
- **Connectives** — truth-table definitions for all standard binary connectives (AND, OR, IMPLIES, IFF, NAND, NOR, XOR) with commutativity and duality.
- **CNF / DNF conversion** — negation normal form (NNF), distribution-based CNF, and **Tseitin encoding** for linear-size equisatisfiable CNF.
- **DPLL SAT solver** — unit propagation, pure literal elimination, backtracking search.
- **Resolution** — clausal resolution with unification for both propositional and predicate literals.
- **Predicate logic** — terms, quantifiers (∀, ∃), substitution (capture-avoiding), unification, and Robinson's unification algorithm.
- **Natural deduction** — proof terms as a typed λ-calculus (Curry-Howard correspondence) with full type-checking.
- **Gödel numbering** — encode propositional and predicate formulas as natural numbers via prime factorization.
- **Agent reasoning** — behavioral contracts with preconditions/postconditions/invariants, SAT-based consistency checking, safety verification, and knowledge-base queries.

## Key Idea

The crate treats logic as **data structures you can compute with**. Formulas are `enum` trees. Proofs are typed λ-terms. SAT solving returns satisfying assignments. Resolution produces explicit proof steps. This makes it possible to embed logical reasoning inside larger systems — for example, verifying that an agent's behavioral contract is consistent and its safety properties hold.

## Install

```toml
[dependencies]
lau-logic-foundations = { git = "https://github.com/SuperInstance/lau-logic-foundations" }
```

Requires `serde` and `nalgebra`.

## Quick Start

### Propositional Logic

```rust
use lau_logic_foundations::{atom, and, or, not, implies, Formula};

let p = atom("p");
let q = atom("q");

// Is (p ∧ q → p) a tautology?
let f = implies(and(p.clone(), q.clone()), p.clone());
assert!(f.is_tautology());

// De Morgan: ¬(p ∧ q) ≡ (¬p ∨ ¬q)
let f1 = not(and(atom("p"), atom("q")));
let f2 = or(not(atom("p")), not(atom("q")));
assert!(Formula::equivalent(&f1, &f2));
```

### SAT Solving (DPLL)

```rust
use lau_logic_foundation::{Cnf, Literal, solve_sat, SatResult};

let cnf = Cnf::new(vec![
    vec![Literal::positive("p"), Literal::positive("q")],
    vec![Literal::negative("p"), Literal::positive("q")],
]);

match solve_sat(&cnf) {
    SatResult::Satisfiable(assignment) => { /* use assignment */ }
    SatResult::Unsatisfiable => { /* no solution */ }
}
```

### Predicate Logic & Unification

```rust
use lau_logic_foundation::{Term, PredFormula, unify, Substitution};

let t1 = Term::Func("f".into(), vec![Term::Var("x".into())]);
let t2 = Term::Func("f".into(), vec![Term::Const("a".into())]);

if let UnificationResult::Success(subst) = unify(&t1, &t2) {
    assert_eq!(subst.map["x"], Term::Const("a".into()));
}
```

### Agent Contracts

```rust
use lau_logic_foundation::{AgentContract, AgentKnowledgeBase, atom, implies};

let mut contract = AgentContract::new("my_agent");
contract.invariant(atom("safe"));
contract.safety(atom("safe"));
assert!(contract.verify_safety()[0].holds);

let mut kb = AgentKnowledgeBase::new();
kb.add_fact(atom("p"));
kb.add_rule(implies(atom("p"), atom("q")));
assert!(kb.query(&atom("q")).proven);
```

## API Reference

### `propositional` — Formula, Var, builders

| Item | Description |
|---|---|
| `Formula` | Enum: Atom, Not, And, Or, Implies, Iff, Top, Bot |
| `atom`, `not`, `and`, `or`, `implies`, `iff` | Builder functions |
| `Formula::truth_table()` | Exhaustive (assignment → result) pairs |
| `Formula::is_tautology()` | True under all assignments |
| `Formula::is_satisfiable()` | True under some assignment |
| `Formula::entails(other)` | `self → other` is a tautology |
| `Formula::equivalent(a, b)` | `a ↔ b` is a tautology |
| `Formula::substitute(var, replacement)` | Variable substitution |

### `connectives` — UnaryConnective, BinaryConnective

Truth-table evaluation for AND, OR, IMPLIES, IFF, NAND, NOR, XOR. Includes `is_commutative()`, `dual()`, and `TruthTableBuilder`.

### `cnf` — CNF, DNF, Tseitin Encoding

| Item | Description |
|---|---|
| `Literal` | Positive or negative variable |
| `Cnf` | Conjunction of clauses (disjunctions of literals) |
| `Dnf` | Disjunction of terms |
| `formula_to_cnf(f)` | Naïve NNF + distribution |
| `TseitinEncoder::tseitin_encode(f)` | Linear equisatisfiable CNF |
| `formula_to_dnf(f)` | Disjunctive normal form |

### `dpll` — DpllSolver

| Item | Description |
|---|---|
| `DpllSolver::new()` | Create solver (with pure literal elimination) |
| `solve(&Cnf)` | Returns `SatResult::Satisfiable(assignment)` or `Unsatisfiable` |
| `solve_sat(cnf)` | Convenience function |

### `predicate` — Term, PredFormula, Substitution

| Item | Description |
|---|---|
| `Term` | Var, Const, Func(name, args) |
| `PredFormula` | Atom, Not, And, Or, Implies, Iff, Forall, Exists, Top, Bot |
| `PredFormula::free_vars()` | Collect free variables |
| `PredFormula::substitute(var, term)` | Capture-avoiding substitution |
| `Substitution` | Variable → term mapping, with `compose()` |
| `unify(t1, t2)` | Robinson unification → `UnificationResult` |
| `unify_list(ts1, ts2)` | Unify parallel term lists |

### `resolution` — ResolutionProver

| Item | Description |
|---|---|
| `ResolutionProver::prove(&[ResClause])` | Refutation by resolution → `bool` |
| `prop_to_res_clauses(&[Clause])` | Convert propositional CNF clauses |

### `natural_deduction` — ProofTerm, ProofContext

| Item | Description |
|---|---|
| `ProofTerm` | Var, Abs, App, Pair, Fst, Snd, Inl, Inr, Case, Abort, Trivial |
| `ProofContext` | Manages assumptions; provides intro/elim rule builders |
| `type_check(ctx, term)` | Returns `Result<Formula, String>` |

### `godel` — Gödel Numbering

| Item | Description |
|---|---|
| `GodelEncoding` | Symbol table with standard codes (¬=1, ∨=2, →=3, ∀=4, ∧=5, ↔=6, ∃=7) |
| `encode_formula(enc, f)` | Propositional formula → Gödel number |
| `encode_pred_formula(enc, f)` | Predicate formula → Gödel number |
| `encode_sequence(&[u64])` | Sequence → prime-factorization Gödel number |
| `decode_sequence(godel_num, len)` | Inverse |
| `beta(b, c, i)` | Gödel's β function |

### `agent` — AgentContract, AgentKnowledgeBase

| Item | Description |
|---|---|
| `AgentContract` | Preconditions, postconditions, invariants, safety properties |
| `check_consistency()` | SAT-based consistency check |
| `verify_safety()` | Check invariants ⊨ safety |
| `AgentKnowledgeBase` | Facts + rules → query by refutation |
| `query(goal)` | Returns `QueryResult { proven, explanation }` |

## How It Works

1. **Formulas as trees.** Both propositional and predicate formulas are recursive `enum` types, enabling pattern-matching traversals for evaluation, substitution, and transformation.

2. **CNF via NNF + distribution.** `formula_to_cnf` first converts to Negation Normal Form (pushing ¬ inward using De Morgan's laws), then distributes ∨ over ∧. This is exponential in the worst case.

3. **Tseitin encoding** avoids the exponential blowup by introducing auxiliary variables for each subformula, producing an *equisatisfiable* (not equivalent) CNF in linear size.

4. **DPLL** operates on the CNF clause list: unit propagation forces single-literal clauses, pure literal elimination assigns variables that appear in only one polarity, and backtracking search branches on remaining variables.

5. **Resolution** finds complementary literals (one positive, one negative) with matching predicates, unifies their arguments, and produces the resolvent. Refutation succeeds when the empty clause is derived.

6. **Natural deduction** uses the Curry-Howard correspondence: proofs are typed λ-terms (abstraction = →-intro, application = →-elim, pairs = ∧-intro, projections = ∧-elim, etc.). `type_check` is the proof checker.

7. **Gödel numbering** encodes formulas as natural numbers using prime factorization: the sequence [a₁, ..., aₙ] maps to 2^a₁ · 3^a₂ · ... · pₙ^aₙ. Each subformula gets its own Gödel number recursively.

8. **Agent reasoning** reduces contract verification to SAT: safety property P holds iff (invariants ∧ preconditions ∧ ¬P) is unsatisfiable. Knowledge-base queries work the same way.

## The Math

**Truth-table semantics:** A formula's meaning is its truth table over all 2ⁿ assignments to its *n* variables.

**CNF:** Every propositional formula φ has an equivalent CNF. Naïve conversion via distribution can produce 2^|φ| clauses; Tseitin encoding produces O(|φ|) clauses at the cost of auxiliary variables.

**DPLL completeness:** For any CNF formula, DPLL returns SAT with a model or UNSAT. The search explores a binary tree of depth ≤ *n* (number of variables), pruned by unit propagation and pure literal elimination.

**Resolution soundness & completeness:** Resolution is sound (every resolvent is a logical consequence) and refutation-complete for first-order logic (though not decidable in general). For propositional logic, it is a decision procedure.

**Unification:** Robinson's algorithm finds the most general unifier (MGU) for two terms by structurally decomposing and composing substitutions. Occurs check is implicit (terms are acyclic by construction).

**Curry-Howard:** Types are propositions; terms are proofs. `Abs` (λ-abstraction) is →-introduction; `App` is →-elimination. Type checking = proof checking.

**Gödel's β function:** β(b, c, i) = c mod (1 + (i+1)·b) encodes/decodes finite sequences as single natural numbers, enabling arithmetization of syntax.

## Tests

**97 unit tests** across 9 modules: propositional evaluation, truth tables, tautology/contradiction/satisfiability, De Morgan equivalence, entailment, CNF/DNF conversion, Tseitin encoding, DPLL (unit propagation, pure literal, backtracking, 3-SAT), literal operations, resolution (propositional & predicate), predicate substitution, free variables, unification, substitution composition, natural deduction (modus ponens, ∧-intro/elim, identity, ∨-intro, ⊥-elim, type errors), Gödel encoding/decoding roundtrips, β function, agent contracts, knowledge base queries.

## License

MIT
