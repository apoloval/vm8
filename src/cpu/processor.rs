#[derive(Default)]
pub struct ExecutionPlan {
    pub max_cycles: Option<usize>,
    pub max_instructions: Option<usize>,
}

impl ExecutionPlan {
    pub fn with_max_cycles(value: usize) -> Self {
        Self { max_cycles: Some(value), ..Default::default() }
    }

    pub fn with_max_instructions(value: usize) -> Self {
        Self { max_instructions: Some(value), ..Default::default() }
    }

    pub fn is_completed(&self, result: &ExecutionResult) -> bool {
        if Self::limit_reached(self.max_cycles, result.total_cycles) {
            return true;
        }
        if Self::limit_reached(self.max_instructions, result.total_instructions) {
            return true;
        }
        false
    }

    fn limit_reached(limit: Option<usize>, value: usize) -> bool {
        match limit {
            Some(max) => value >= max,
            None => false,
        }
    }
}

#[derive(Default)]
pub struct ExecutionResult {
    pub total_cycles: usize,
    pub total_instructions: usize,
}

// A executor that can execute instructions according to a plan.
pub trait Processor {
    // Execute instructions according to the given plan.
    fn execute(&mut self, plan: &ExecutionPlan) -> ExecutionResult;
}
