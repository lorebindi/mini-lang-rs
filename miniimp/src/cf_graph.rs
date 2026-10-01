//! Control Flow Graph (CFG) representations and block reduction algorithms for 'miniimp'.
//!
//! This module models programs as directed graphs of basic blocks according to the formal
//! definition (N, next, i, f, code).

use crate::ast::{BoolExpr, ArithExpr, Cmd};

/// Unique strongly-typed identifier for a 'BasicBlock' in a 'CfGraph'.
/// Wraps a 0-based 'usize' indexing into the contiguous storage vectors of the parent graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);


/// Elementary instruction primitive within a 'BasicBlock'.
///
/// - 'BasicStmt::Skip': Identity / no-op instruction used for control alignment and branch joins.
/// - 'BasicStmt::Assign': Atomic variable mutation binding an arithmetic expression evaluation.
/// - 'BasicStmt::Condition': Guard expression evaluated before multi-way conditional branching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BasicStmt {
    Skip,
    Assign(String, ArithExpr),
    Condition(BoolExpr),
}

/// A sequence of straight-line 'BasicStmt' executed sequentially without branching.
///
/// In minimal CFG mode, each block contains exactly one statement. In maximal CFG mode,
/// linear sequences of non-branching instructions are stored into a single block.
#[derive(Debug)]
pub struct BasicBlock {
    pub stmts: Vec<BasicStmt>,
}

impl BasicBlock {

    pub fn new() -> Self {
        Self { stmts: vec![] }
    }

    pub fn add_stmt(&mut self, stmt: BasicStmt) {
        self.stmts.push(stmt);
    }
}

/// Ephemeral sub-graph fragment generated during recursive AST-to-CFG lowering.
///
/// Encapsulates the entry node and the dangling exit boundary of an evaluated command sub-tree,
/// allowing caller constructs (such as sequential composition or loops) to wire inbound and
/// outbound control flow edges.
struct Fragment {
    entry: BlockId,
    exit: BlockId,   // the block from which the flow continues
}

/// Explicit Control Flow Graph representation using parallel adjacency vectors.
///
/// Encapsulates the 5-tuple (N, next, i, f, code):
/// - 'blocks': Storage for block payloads representing nodes (N) and their statements (code(n)).
/// - 'succ': Forward adjacency lists representing the (next) edge relation.
/// - 'pred': Inverted adjacency lists tracking incoming predecessors for flow analysis and merging.
/// - 'entry': Unique initial node.
/// - 'exit': Unique terminal sink node.
pub struct CfGraph {
    blocks: Vec<BasicBlock>,
    succ: Vec<Vec<BlockId>>,  // succ[b] = successors of b
    pred: Vec<Vec<BlockId>>,  // predecessors
    entry: BlockId,
    exit: BlockId,
}

impl CfGraph {

    fn empty () -> Self {
        Self {
            blocks: Vec::new(),
            succ: Vec::new(),
            pred: Vec::new(),
            entry: BlockId(0), // placeholder
            exit: BlockId(0),
        }
    }

    // Returns the BlockId of the entry of the cfg
    pub fn entry(&self) -> BlockId { self.entry }
    // Returns the BlockId of the exit of the cfg
    pub fn exit(&self) -> BlockId { self.exit }
    // Returns a slice on the Vec of successors of b
    pub fn successors(&self, b: BlockId) -> &[BlockId] { &self.succ[b.0] }
    // Returns a slice on the Vec of predecessors of b
    pub fn predecessors(&self, b: BlockId) -> &[BlockId] { &self.pred[b.0] }
    // Returns a reference to the BasicBlock with 'b' as BlockId
    pub fn block(&self, b: BlockId) -> &BasicBlock { &self.blocks[b.0] }
    // Returns the number of blocks in the current cfg
    pub fn num_blocks(&self) -> usize { self.blocks.len() }

    // Add an empty block to the cfg
    fn add_empty_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len());
        self.blocks.push(BasicBlock::new());
        self.succ.push(Vec::new());
        self.pred.push(Vec::new());
        id
    }

    // Add a block to the cfg with a specific stmt
    fn add_block(&mut self, stmt: BasicStmt) -> BlockId {
        let id = self.add_empty_block();
        self.blocks[id.0].add_stmt(stmt);
        id
    }

    // Add an edge between 'from' and 'to' BlockIds
    fn add_edge(&mut self, from: BlockId, to: BlockId) {
        self.succ[from.0].push(to);
        self.pred[to.0].push(from);
    }

    /// Starting from a Cmd it updates the cfg structure and returns the fragment
    fn build_fragment (&mut self, cmd: &Cmd) -> Fragment {
        match cmd {
            Cmd::Skip => {
                let b = self.add_block(BasicStmt::Skip);
                Fragment { entry: b, exit: b }
            }
            Cmd::Assign(lhs, rhs) => {
                let b = self.add_block(BasicStmt::Assign(lhs.clone(), rhs.clone()));
                Fragment { entry: b, exit: b }
            }
            Cmd::Seq(lhs, rhs) => {
                let frag1 = self.build_fragment(lhs);
                let frag2 = self.build_fragment(rhs);
                self.add_edge(frag1.exit, frag2.entry);
                Fragment { entry: frag1.entry, exit: frag2.exit }
            }
            Cmd::If(cond, c1, c2) => {
                // Add condition block
                let b_cond = self.add_block(BasicStmt::Condition(cond.clone()));
                // Build the fragment of the branches
                let frag1 = self.build_fragment(c1);
                let frag2 = self.build_fragment(c2);
                // Links the condition blocks to the branches fragments
                self.add_edge(b_cond, frag1.entry);
                self.add_edge(b_cond, frag2.entry);
                // Add join (Skip) block of the if
                let b_skip = self.add_block(BasicStmt::Skip);
                // Links the branches fragments to the join block
                self.add_edge(frag1.exit, b_skip);
                self.add_edge(frag2.exit, b_skip);
                Fragment { entry: b_cond, exit: b_skip }
            }
            Cmd::While(cond, c) => {
                // Add condition block
                let b_cond = self.add_block(BasicStmt::Condition(cond.clone()));
                // Build the fragment of the body
                let frag_c = self.build_fragment(c);
                // Add join (Skip) block of the while
                let b_skip = self.add_block(BasicStmt::Skip);
                // Links the condition block to the body fragment
                self.add_edge(b_cond, frag_c.entry);   // true
                // Links the body fragment to the condition block
                self.add_edge(frag_c.exit, b_cond);    // back-edge
                // Links the condition block to the join block
                self.add_edge(b_cond, b_skip);         // false
                Fragment { entry: b_cond, exit: b_skip }
            }
            Cmd::Bracket(c) => {
                self.build_fragment(c)
            }
        }
    }

    /// Return the cfg with minimal blocks starting from the ast
    pub fn from_ast_minimal(cmd: &Cmd) -> Self{
        let mut cfg = CfGraph::empty();
        let entry = cfg.add_block(BasicStmt::Skip);
        let frag = cfg.build_fragment(cmd);
        cfg.add_edge(entry, frag.entry);
        cfg.entry = entry;
        cfg.exit = frag.exit;
        cfg
    }

    /// Return true if the current cfg uses a minimal blocks
    pub fn is_minimal(&self) -> bool { self.blocks.iter().all(|b| b.stmts.len() == 1) }
    // Return true if the current cfg uses maximal blocks
    pub fn is_maximal(&self) -> bool {
        (0..self.blocks.len()).all(|i| self.merge_target(BlockId(i)).is_none())
    }

    /// Return Some(p) if 's' can be absorbed into its unique predecessor 'p', i.e. is not a chain head.
    fn merge_target(&self, s: BlockId) -> Option<BlockId> {
        if s == self.entry { return None; }
        // if 's' has a single predecessor good, otherwise 'None'
        let [p] = self.pred[s.0][..] else { return None };
        if p == s || p == self.entry {
            // The second part of this condition don't allow that the original
            // entry (fictitious Skip) will be merged with the successor even if
            // the merge could be applied
            return None;
        }
        // if 'p' has 's' as unique predecessor then 'Some(p)'
        (self.succ[p.0].len() == 1).then_some(p)
    }

    /// Return an equivalent cfg that is built with maximal blocks
    ///
    /// First it found the heads of minimal blocks chains, then starting
    /// from each head merge it with all the successors that can be absorbed.
    /// Finally links the new maximal blocks following the exit edges of the
    /// last absorbed successor provided by the original cfg.
    ///
    /// Note: the fictitious Skip introduced as join for the If and the While,
    /// is not filtered during the merge, it will remain as a first stmt in
    /// the merged block that, possibly contains join's successors.
    pub fn to_maximal_blocks(&self) -> CfGraph {
        let mut out = CfGraph::empty(); // no blocks yet
        let mut new_id: Vec<Option<BlockId>> = vec![None; self.blocks.len()]; // mapping old BlockIds to new BlockIds
        let mut tails = Vec::new(); // (new block, last old block of its chain)

        for i in 0..self.blocks.len() {
            // we are looking for the heads of minimal blocks chains
            let head = BlockId(i);
            if self.merge_target(head).is_some() { continue; }   // not a chain head

            let new_max_block = out.add_empty_block();
            let mut cur = head;

            loop {
                // storing in the head's Id the new maximal block
                new_id[cur.0] = Some(new_max_block);
                // copy the stmts of the head in the new maximal block
                out.blocks[new_max_block.0].stmts.extend(self.blocks[cur.0].stmts.iter().cloned());
                match self.succ[cur.0][..] {
                    // if the successor can be absorbed continue
                    [s] if self.merge_target(s).is_some() => cur = s,
                    // otherwise the chain is ended
                    _ => break,
                }
            }
            tails.push((new_max_block, cur));
        }

        // Edges only after every chain has an id.
        for (new_max_block, tail) in tails {
            for &s in &self.succ[tail.0] {
                out.add_edge(new_max_block, new_id[s.0].unwrap());
            }
        }

        out.entry = new_id[self.entry.0].unwrap();
        out.exit = new_id[self.exit.0].unwrap();
        out
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ArithExpr, BoolExpr, Cmd};

    // ---------- base cases ----------

    // skip
    #[test]
    fn single_skip() {
        let cfg = CfGraph::from_ast_minimal(&Cmd::Skip);
        assert!(cfg.is_minimal());
        // entry(Skip) -> body(Skip) == exit
        let succ_entry = cfg.successors(cfg.entry());
        assert_eq!(succ_entry.len(), 1);
        let body = succ_entry[0];
        assert_eq!(body, cfg.exit());
        assert!(matches!(cfg.block(body).stmts[0], BasicStmt::Skip));
    }

    // x := 1
    #[test]
    fn single_assign() {
        let cmd = Cmd::Assign("x".to_string(), ArithExpr::Num(1));
        let cfg = CfGraph::from_ast_minimal(&cmd);
        assert!(cfg.is_minimal());
        let succ_entry = cfg.successors(cfg.entry());
        assert_eq!(succ_entry.len(), 1);
        let body = succ_entry[0];
        assert_eq!(body, cfg.exit());
        assert!(matches!(cfg.block(body).stmts[0], BasicStmt::Assign(ref n, _) if n == "x"));
    }

    // (x := 1) with Bracket: must produce the same graph as x := 1
    #[test]
    fn bracket_is_transparent() {
        let cmd = Cmd::Bracket(Box::new(Cmd::Assign("x".to_string(), ArithExpr::Num(1))));
        let cfg = CfGraph::from_ast_minimal(&cmd);
        assert!(cfg.is_minimal());
        assert_eq!(cfg.num_blocks(), 2); // entry + 1 blocco
        let succ_entry = cfg.successors(cfg.entry());
        assert_eq!(succ_entry.len(), 1);
        assert_eq!(succ_entry[0], cfg.exit());
    }

    // if true then (x := 1) else (y := 2)
    #[test]
    fn simple_if_then_else() {
        let cmd = Cmd::If(
            BoolExpr::True,
            Box::new(Cmd::Assign("x".to_string(), ArithExpr::Num(1))),
            Box::new(Cmd::Assign("y".to_string(), ArithExpr::Num(2))),
        );
        let cfg = CfGraph::from_ast_minimal(&cmd);
        assert!(cfg.is_minimal());

        let cond = cfg.successors(cfg.entry())[0];
        assert!(matches!(cfg.block(cond).stmts[0], BasicStmt::Condition(_)));

        let succ_cond = cfg.successors(cond);
        assert_eq!(succ_cond.len(), 2);
        let (b1, b2) = (succ_cond[0], succ_cond[1]);
        assert!(matches!(cfg.block(b1).stmts[0], BasicStmt::Assign(ref n, _) if n == "x"));
        assert!(matches!(cfg.block(b2).stmts[0], BasicStmt::Assign(ref n, _) if n == "y"));

        // entrambi convergono sull'exit
        assert_eq!(cfg.successors(b1), &[cfg.exit()]);
        assert_eq!(cfg.successors(b2), &[cfg.exit()]);
        assert_eq!(cfg.predecessors(cfg.exit()).len(), 2);
    }

    // Sequence: x := 1; y := 2
    // Expected: entry(Skip) -> b1(Assign x) -> b2(Assign y) == exit
    #[test]
    fn two_statement_sequence() {
        let cmd = Cmd::Seq(
            Box::new(Cmd::Assign("x".to_string(), ArithExpr::Num(1))),
            Box::new(Cmd::Assign("y".to_string(), ArithExpr::Num(2))),
        );
        let cfg = CfGraph::from_ast_minimal(&cmd);

        assert!(cfg.is_minimal());

        // invariats: entry doesn't have predecessors, exit doesn't have successors
        assert!(cfg.predecessors(cfg.entry()).is_empty());
        assert!(cfg.successors(cfg.exit()).is_empty());

        // entry -> b1
        let succ_entry = cfg.successors(cfg.entry());
        assert_eq!(succ_entry.len(), 1);
        let b1 = succ_entry[0];
        assert!(matches!(cfg.block(b1).stmts[0], BasicStmt::Assign(ref n, _) if n == "x"));

        // b1 -> b2
        let succ_b1 = cfg.successors(b1);
        assert_eq!(succ_b1.len(), 1);
        let b2 = succ_b1[0];
        assert!(matches!(cfg.block(b2).stmts[0], BasicStmt::Assign(ref n, _) if n == "y"));

        // b2 == exit
        assert_eq!(b2, cfg.exit());
    }

    // x := 1; y := 2; z := 3 (three commands sequence)
    #[test]
    fn three_statement_sequence() {
        let cmd = Cmd::Seq(
            Box::new(Cmd::Seq(
                Box::new(Cmd::Assign("x".to_string(), ArithExpr::Num(1))),
                Box::new(Cmd::Assign("y".to_string(), ArithExpr::Num(2))),
            )),
            Box::new(Cmd::Assign("z".to_string(), ArithExpr::Num(3))),
        );
        let cfg = CfGraph::from_ast_minimal(&cmd);
        assert!(cfg.is_minimal());
        assert_eq!(cfg.num_blocks(), 4); // entry + 3 assegnamenti

        let b1 = cfg.successors(cfg.entry())[0];
        let b2 = cfg.successors(b1)[0];
        let b3 = cfg.successors(b2)[0];
        assert_eq!(b3, cfg.exit());
        assert!(matches!(cfg.block(b1).stmts[0], BasicStmt::Assign(ref n, _) if n == "x"));
        assert!(matches!(cfg.block(b2).stmts[0], BasicStmt::Assign(ref n, _) if n == "y"));
        assert!(matches!(cfg.block(b3).stmts[0], BasicStmt::Assign(ref n, _) if n == "z"));
    }

    // if true then x:=1 else y:=2; if true then a:=1 else b:=2
    // (a sequence of two if to verify that the join of the first links
    // correctly to the entry of the second)
    #[test]
    fn two_ifs_in_sequence() {
        let cmd = Cmd::Seq(
            Box::new(Cmd::If(
                BoolExpr::True,
                Box::new(Cmd::Assign("x".to_string(), ArithExpr::Num(1))),
                Box::new(Cmd::Assign("y".to_string(), ArithExpr::Num(2))),
            )),
            Box::new(Cmd::If(
                BoolExpr::True,
                Box::new(Cmd::Assign("a".to_string(), ArithExpr::Num(1))),
                Box::new(Cmd::Assign("b".to_string(), ArithExpr::Num(2))),
            )),
        );
        let cfg = CfGraph::from_ast_minimal(&cmd);
        assert!(cfg.is_minimal());

        let cond1 = cfg.successors(cfg.entry())[0];
        let succ1 = cfg.successors(cond1);
        assert_eq!(succ1.len(), 2);
        let join1 = cfg.successors(succ1[0])[0];
        assert_eq!(join1, cfg.successors(succ1[1])[0]);

        // join1 links to the second if
        let succ_join1 = cfg.successors(join1);
        assert_eq!(succ_join1.len(), 1);
        let cond2 = succ_join1[0];
        assert!(matches!(cfg.block(cond2).stmts[0], BasicStmt::Condition(_)));

        let succ2 = cfg.successors(cond2);
        assert_eq!(succ2.len(), 2);
        let join2 = cfg.successors(succ2[0])[0];
        assert_eq!(join2, cfg.successors(succ2[1])[0]);
        assert_eq!(join2, cfg.exit());
    }

    // while true do skip
    // expected: entry -> cond -> body -> cond (back-edge), cond -> exit(Skip)
    #[test]
    fn while_forms_a_back_edge_and_entry_has_no_incoming() {
        let cmd = Cmd::While(
            BoolExpr::True,
            Box::new(Cmd::Skip),
        );
        let cfg = CfGraph::from_ast_minimal(&cmd);

        assert!(cfg.is_minimal());

        // invariats: entry doesn't have predecessors, exit doesn't have successors
        assert!(cfg.predecessors(cfg.entry()).is_empty());
        assert!(cfg.successors(cfg.exit()).is_empty());

        // entry -> cond
        let succ_entry = cfg.successors(cfg.entry());
        assert_eq!(succ_entry.len(), 1);
        let cond = succ_entry[0];
        assert!(matches!(cfg.block(cond).stmts[0], BasicStmt::Condition(_)));

        // cond has two successors: body (true) and exit (false)
        let succ_cond = cfg.successors(cond);
        assert_eq!(succ_cond.len(), 2);
        let body = succ_cond[0];
        let after = succ_cond[1];
        assert_eq!(after, cfg.exit());

        // body -> cond (back-edge)
        let succ_body = cfg.successors(body);
        assert_eq!(succ_body, &[cond]);

        // cond has two successors: entry and body
        assert_eq!(cfg.predecessors(cond).len(), 2);
    }

    // while true do (if true then skip else skip)
    // Expected:
    //   entry -> cond_w
    //   cond_w -> cond_i (true), cond_w -> exit (false)
    //   cond_i -> b1 (true), cond_i -> b2 (false)
    //   b1 -> join, b2 -> join
    //   join -> cond_w (back-edge)
    #[test]
    fn if_inside_while_joins_before_the_back_edge() {
        let cmd = Cmd::While(
            BoolExpr::True,
            Box::new(Cmd::If(
                BoolExpr::True,
                Box::new(Cmd::Skip),
                Box::new(Cmd::Skip),
            )),
        );
        let cfg = CfGraph::from_ast_minimal(&cmd);

        assert!(cfg.is_minimal());
        assert!(cfg.predecessors(cfg.entry()).is_empty());
        assert!(cfg.successors(cfg.exit()).is_empty());

        // entry -> cond_w
        let succ_entry = cfg.successors(cfg.entry());
        assert_eq!(succ_entry.len(), 1);
        let cond_w = succ_entry[0];
        assert!(matches!(cfg.block(cond_w).stmts[0], BasicStmt::Condition(_)));

        // cond_w -> cond_i (true), cond_w -> exit (false)
        let succ_cond_w = cfg.successors(cond_w);
        assert_eq!(succ_cond_w.len(), 2);
        let cond_i = succ_cond_w[0];
        let after_loop = succ_cond_w[1];
        assert_eq!(after_loop, cfg.exit());
        assert!(matches!(cfg.block(cond_i).stmts[0], BasicStmt::Condition(_)));

        // cond_i -> b1 (true), cond_i -> b2 (false), both Skip
        let succ_cond_i = cfg.successors(cond_i);
        assert_eq!(succ_cond_i.len(), 2);
        let b1 = succ_cond_i[0];
        let b2 = succ_cond_i[1];
        assert!(matches!(cfg.block(b1).stmts[0], BasicStmt::Skip));
        assert!(matches!(cfg.block(b2).stmts[0], BasicStmt::Skip));

        // b1 and b2 converge on the same join
        let succ_b1 = cfg.successors(b1);
        let succ_b2 = cfg.successors(b2);
        assert_eq!(succ_b1.len(), 1);
        assert_eq!(succ_b2.len(), 1);
        let join = succ_b1[0];
        assert_eq!(join, succ_b2[0]);

        // join has two predecessors (b1, b2)
        assert_eq!(cfg.predecessors(join).len(), 2);

        // join -> cond_w: back-edge
        assert_eq!(cfg.successors(join), &[cond_w]);

        // cond_w has two predecessors: entry and join
        assert_eq!(cfg.predecessors(cond_w).len(), 2);
    }

    // while true do (while true do skip): while nested in while
    #[test]
    fn nested_while() {
        let cmd = Cmd::While(
            BoolExpr::True,
            Box::new(Cmd::While(BoolExpr::True, Box::new(Cmd::Skip))),
        );
        let cfg = CfGraph::from_ast_minimal(&cmd);
        assert!(cfg.is_minimal());

        let cond_outer = cfg.successors(cfg.entry())[0];
        let succ_outer = cfg.successors(cond_outer);
        assert_eq!(succ_outer.len(), 2);
        let cond_inner = succ_outer[0];
        assert_eq!(succ_outer[1], cfg.exit());
        assert!(matches!(cfg.block(cond_inner).stmts[0], BasicStmt::Condition(_)));

        let succ_inner = cfg.successors(cond_inner);
        assert_eq!(succ_inner.len(), 2);
        let body_inner = succ_inner[0];
        let after_inner = succ_inner[1]; // internal while exit is an isolated block

        // back-edge of the internal while
        assert_eq!(cfg.successors(body_inner), &[cond_inner]);

        // The exit of the internal while return to the external condition (external back-edge)
        assert_eq!(cfg.successors(after_inner), &[cond_outer]);
        assert_eq!(cfg.predecessors(cond_outer).len(), 2); // entry + after_inner
    }

    // ---------- to_maximal_blocks ----------

    // single assign: the entry remains isolated even in the simple case
    #[test]
    fn maximal_single_assign_entry_stays_isolated() {
        let cmd = Cmd::Assign("x".to_string(), ArithExpr::Num(1));
        let minimal = CfGraph::from_ast_minimal(&cmd);
        let maximal = minimal.to_maximal_blocks();
        assert!(maximal.is_maximal());
        assert_eq!(maximal.num_blocks(), 2); // isolated entry + x block
        let body = maximal.successors(maximal.entry())[0];
        assert_eq!(body, maximal.exit());
        assert_eq!(maximal.block(body).stmts.len(), 1);
    }

    // The chain entry(skip) -> x := 1 -> y := 2 -> z := 3 (exit) must merge
    // in a unique block [x, y, z], while the entry remain isolated (design
    // choice of merge_target).
    #[test]
    fn maximal_merges_linear_chain_but_keeps_entry_alone() {
        let cmd = Cmd::Seq(
            Box::new(Cmd::Seq(
                Box::new(Cmd::Assign("x".to_string(), ArithExpr::Num(1))),
                Box::new(Cmd::Assign("y".to_string(), ArithExpr::Num(2))),
            )),
            Box::new(Cmd::Assign("z".to_string(), ArithExpr::Num(3))),
        );
        let minimal = CfGraph::from_ast_minimal(&cmd);
        let maximal = minimal.to_maximal_blocks();

        assert!(maximal.is_maximal());
        assert_eq!(maximal.num_blocks(), 2); // isolated entry + merged block

        let body = maximal.successors(maximal.entry())[0];
        assert_eq!(body, maximal.exit());
        assert_eq!(maximal.block(body).stmts.len(), 3);
        assert!(matches!(maximal.block(body).stmts[0], BasicStmt::Assign(ref n, _) if n == "x"));
        assert!(matches!(maximal.block(body).stmts[1], BasicStmt::Assign(ref n, _) if n == "y"));
        assert!(matches!(maximal.block(body).stmts[2], BasicStmt::Assign(ref n, _) if n == "z"));
    }

    // if true then x:=1 else y:=2; z:=3
    // The join of the if has a unique successor (z) and z has a unique predecessor
    // (the join), hence they must be merged even if the join it's not a "head".
    #[test]
    fn maximal_merges_join_forward_into_next_block() {
        let cmd = Cmd::Seq(
            Box::new(Cmd::If(
                BoolExpr::True,
                Box::new(Cmd::Assign("x".to_string(), ArithExpr::Num(1))),
                Box::new(Cmd::Assign("y".to_string(), ArithExpr::Num(2))),
            )),
            Box::new(Cmd::Assign("z".to_string(), ArithExpr::Num(3))),
        );
        let minimal = CfGraph::from_ast_minimal(&cmd);
        let maximal = minimal.to_maximal_blocks();
        assert!(maximal.is_maximal());

        let cond = maximal.successors(maximal.entry())[0];
        let succ_cond = maximal.successors(cond);
        assert_eq!(succ_cond.len(), 2);
        let (b1, b2) = (succ_cond[0], succ_cond[1]);

        // b1, b2 remains isolated: each of them has a predecessor (cond) with succ.len()==2
        assert_eq!(maximal.block(b1).stmts.len(), 1);
        assert_eq!(maximal.block(b2).stmts.len(), 1);

        // the block after the join contains the Skip of the join merged with z
        let after = maximal.successors(b1)[0];
        assert_eq!(after, maximal.successors(b2)[0]);
        assert_eq!(after, maximal.exit());
        assert_eq!(maximal.block(after).stmts.len(), 2);
        assert!(matches!(maximal.block(after).stmts[0], BasicStmt::Skip));
        assert!(matches!(maximal.block(after).stmts[1], BasicStmt::Assign(ref n, _) if n == "z"));
    }


    // while true do (a := 1; b := 2): the two assignment body must be merged
    // into a single block; the condition remain isolated with 2 succ/pred.
    #[test]
    fn maximal_merges_while_body() {
        let cmd = Cmd::While(
            BoolExpr::True,
            Box::new(Cmd::Seq(
                Box::new(Cmd::Assign("a".to_string(), ArithExpr::Num(1))),
                Box::new(Cmd::Assign("b".to_string(), ArithExpr::Num(2))),
            )),
        );
        let minimal = CfGraph::from_ast_minimal(&cmd);
        let maximal = minimal.to_maximal_blocks();

        assert!(maximal.is_maximal());

        let cond = maximal.successors(maximal.entry())[0];
        assert!(matches!(maximal.block(cond).stmts[0], BasicStmt::Condition(_)));

        let succ_cond = maximal.successors(cond);
        assert_eq!(succ_cond.len(), 2);
        let body = succ_cond[0];
        assert_eq!(succ_cond[1], maximal.exit());

        // merged body: two assignment in the same body
        assert_eq!(maximal.block(body).stmts.len(), 2);
        assert!(matches!(maximal.block(body).stmts[0], BasicStmt::Assign(ref n, _) if n == "a"));
        assert!(matches!(maximal.block(body).stmts[1], BasicStmt::Assign(ref n, _) if n == "b"));

        // back-edge: body -> cond
        assert_eq!(maximal.successors(body), &[cond]);
        assert_eq!(maximal.predecessors(cond).len(), 2); // entry + body
    }

    // while true do (if true then skip else skip): the join remains
    // a separated block because it has two predecessors, also in the maximal cfg.
    #[test]
    fn maximal_keeps_join_separate() {
        let cmd = Cmd::While(
            BoolExpr::True,
            Box::new(Cmd::If(
                BoolExpr::True,
                Box::new(Cmd::Skip),
                Box::new(Cmd::Skip),
            )),
        );
        let minimal = CfGraph::from_ast_minimal(&cmd);
        let maximal = minimal.to_maximal_blocks();

        assert!(maximal.is_maximal());

        let cond_w = maximal.successors(maximal.entry())[0];
        let cond_i = maximal.successors(cond_w)[0];
        let succ_cond_i = maximal.successors(cond_i);
        assert_eq!(succ_cond_i.len(), 2);
        let (b1, b2) = (succ_cond_i[0], succ_cond_i[1]);

        // b1, b2 remains isolated blocks (each of them contains a single stmt, Skip)
        assert_eq!(maximal.block(b1).stmts.len(), 1);
        assert_eq!(maximal.block(b2).stmts.len(), 1);

        let join = maximal.successors(b1)[0];
        assert_eq!(join, maximal.successors(b2)[0]);
        assert_eq!(maximal.predecessors(join).len(), 2); // cannot be merged back
        assert_eq!(maximal.successors(join), &[cond_w]);  // back-edge untouched
    }

    // while true do (while true do (a:=1; b:=2)): nested while, internal body merged
    #[test]
    fn maximal_merges_nested_while_body() {
        let cmd = Cmd::While(
            BoolExpr::True,
            Box::new(Cmd::While(
                BoolExpr::True,
                Box::new(Cmd::Seq(
                    Box::new(Cmd::Assign("a".to_string(), ArithExpr::Num(1))),
                    Box::new(Cmd::Assign("b".to_string(), ArithExpr::Num(2))),
                )),
            )),
        );
        let minimal = CfGraph::from_ast_minimal(&cmd);
        let maximal = minimal.to_maximal_blocks();
        assert!(maximal.is_maximal());

        let cond_outer = maximal.successors(maximal.entry())[0];
        let succ_outer = maximal.successors(cond_outer);
        assert_eq!(succ_outer.len(), 2);
        let cond_inner = succ_outer[0];
        assert_eq!(succ_outer[1], maximal.exit());

        let succ_inner = maximal.successors(cond_inner);
        assert_eq!(succ_inner.len(), 2);
        let body_inner = succ_inner[0];
        let after_inner = succ_inner[1]; // exit of the internal while isolated

        // internal body merged in a unique block
        assert_eq!(maximal.block(body_inner).stmts.len(), 2);
        assert_eq!(maximal.successors(body_inner), &[cond_inner]); // back-edge interno

        // after_inner doesn't merge neither back nor next
        assert_eq!(maximal.block(after_inner).stmts.len(), 1);
        assert_eq!(maximal.successors(after_inner), &[cond_outer]);
        assert_eq!(maximal.predecessors(cond_outer).len(), 2); // entry + after_inner
    }

    // Idempotency: apply to_maximal_blocks to a maximal cfg
    // must not change the structure.
    #[test]
    fn to_maximal_blocks_is_idempotent() {
        let cmd = Cmd::While(
            BoolExpr::True,
            Box::new(Cmd::Seq(
                Box::new(Cmd::Assign("a".to_string(), ArithExpr::Num(1))),
                Box::new(Cmd::Assign("b".to_string(), ArithExpr::Num(2))),
            )),
        );
        let minimal = CfGraph::from_ast_minimal(&cmd);
        let once = minimal.to_maximal_blocks();
        let twice = once.to_maximal_blocks();

        assert!(once.is_maximal());
        assert!(twice.is_maximal());
        assert_eq!(once.num_blocks(), twice.num_blocks());
    }
}

