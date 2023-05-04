use serde::{Deserialize, Serialize};

use crate::{
    clocks::{S4Vector, VectorClock},
    rga::{RGA, SnapshotIter},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertOperation {
    character: char,
    insert_after: [u32; 4],
    insert_position: [u32; 4],
    op_clock: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OperationData {
    Insert(InsertOperation),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Operation {
    sent_by: usize,
    data: OperationData,
}

pub struct SynchronizedText {
    clock: VectorClock,
    rga: RGA<char>,
}

impl SynchronizedText {
    pub fn new(id: usize) -> SynchronizedText {
        SynchronizedText {
            clock: VectorClock::new(id),
            rga: RGA::new(),
        }
    }
    pub fn get_text(&self) -> String {
        self.rga
            .iter()
            .filter(|(_, c)| c.is_some())
            .map(|(_, c)| c.unwrap())
            .collect()
    }

    pub fn get_positions(&self) -> Vec<S4Vector> {
        self.rga.iter().map(|(p, _)| p).collect()
    }

    pub fn local_insert(&mut self, insert_after: S4Vector, character: char) -> Operation {
        self.clock.increase();
        self.rga
            .insert(insert_after, self.clock.to_s4vector(), character);
        Operation {
            sent_by: self.clock.id(),
            data: OperationData::Insert(InsertOperation {
                character,
                insert_after: insert_after.to_array(),
                insert_position: self.clock.to_s4vector().to_array(),
                op_clock: self.clock.cloned_values(),
            }),
        }
    }

    pub fn remote_insert(
        &mut self,
        sent_by: usize,
        sent_clock_values: &[u32],
        operation_position: S4Vector,
        insert_after: S4Vector,
        character: char,
    ) -> Result<(), String> {
        if !self.is_ready_to_receive(sent_by, sent_clock_values) {
            // Normally we would enqueue this operation and wait until the previous values would arrive
            return Err("Not ready to receive this values".into());
        }
        self.rga.insert(insert_after, operation_position, character);
        self.clock.merge_remote(sent_clock_values);

        Ok(())
    }

    pub fn is_ready_to_receive(&self, sent_by: usize, sent_clock_values: &[u32]) -> bool {
        for idx in 0..sent_clock_values.len() {
            // check if any of the clock values is larger than the arriving one
            if sent_clock_values[idx] > self.clock.clock_value(idx) && idx != sent_by {
                return false;
            }
        }

        let sent_clock_value: u32 = *sent_clock_values
            .get(sent_by)
            .expect("The sender must have the value for it's own id");
        sent_clock_value == self.clock.clock_value(sent_by) + 1
    }

    pub fn get_clock(&self) -> &VectorClock {
        &self.clock
    }

    pub fn iter(&self) -> SnapshotIter<'_, char> {
        self.rga.iter()
    }

    pub fn apply_operation(&mut self, operation: Operation) -> Result<(), String> {
        match operation.data {
            OperationData::Insert(op) => self.remote_insert(
                operation.sent_by,
                &op.op_clock,
                op.insert_position.into(),
                op.insert_after.into(),
                op.character,
            )?,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_two_strings() {
        let text1 = "hello ";
        let text2 = "world";
        let mut sync1 = SynchronizedText::new(1);
        let mut sync2 = SynchronizedText::new(0);
        let mut text1_ops = vec![];
        let mut text2_ops = vec![];
        let mut clk = S4Vector::root();
        for c in text1.chars() {
            text1_ops.push(sync1.local_insert(clk, c));
            clk = sync1.clock.to_s4vector();
        }
        clk = S4Vector::root();
        for c in text2.chars() {
            text2_ops.push(sync2.local_insert(clk, c));
            clk = sync2.clock.to_s4vector();
        }

        for op in text1_ops {
            sync2.apply_operation(op).expect("No causality should fail");
        }
        for op in text2_ops {
            sync1.apply_operation(op).expect("No causality should fail");
        }

        assert_eq!(sync1.get_text(), "hello world");
        assert_eq!(sync2.get_text(), "hello world");
    }
}
