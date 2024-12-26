use crate::mir::Function;

impl Function {
    pub fn analyse(mut self) -> Self {
        self.compute_predecessors();
        self.compute_successors();
        self.compute_postorder(); // Depends on successors.
        self.compute_dominators(); // Depends on predecessors.
        self.compute_liveness();
        self
    }

    pub fn with_predecessors(mut self) -> Self {
        self.compute_predecessors();
        self
    }

    pub fn with_successors(mut self) -> Self {
        self.compute_successors();
        self
    }

    pub fn with_postorder(mut self) -> Self {
        self.compute_postorder();
        self
    }

    pub fn with_reverse_postorder_number(mut self) -> Self {
        self.compute_reverse_postorder_number();
        self
    }

    pub fn with_dominators(mut self) -> Self {
        self.compute_dominators();
        self
    }

    pub fn with_liveness(mut self) -> Self {
        self.compute_liveness();
        self
    }
}
