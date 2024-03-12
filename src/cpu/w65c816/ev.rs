
/* An event reported by the CPU. */
#[derive(Debug)]
pub enum Event {
    Exec {
        pbr: u8,
        pc: u16,
        instruction: String,
        operands: String,
    }
}

pub trait Reporter {
    fn report(&mut self, event: impl FnOnce() -> Event);
}

pub struct NullReporter;

impl Reporter for NullReporter {
    fn report(&mut self, _event: impl FnOnce() -> Event) {}
}

#[cfg(test)]
pub struct Retain {
    last: Option<Event>,
}

#[cfg(test)]
impl Retain {
    pub fn new() -> Retain {
        Retain { last: None }
    }

    pub fn assert_exec(&mut self, expected_inst: &str, expected_ops: &str) {
        match self.last {
            Some(Event::Exec { ref instruction, ref operands, ..}) => {
                assert_eq!(expected_inst, instruction);
                assert_eq!(expected_ops, operands);
            }
            _ => panic!("Expected Event::Exec, got {:?}", self.last),
        }
    }
}

#[cfg(test)]
impl Reporter for Retain {
    fn report(&mut self, event: impl FnOnce() -> Event) {
        self.last = Some(event());
    }
}

