extern crate rand;

use rand::distributions::Standard;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::{distributions::Alphanumeric, prelude::*};

use crdt::{
    clocks::S4Vector,
    data_structure::{Operation, SynchronizedText},
};

struct FuzzSuite {
    data_structures: Vec<SynchronizedText>,
    pushed_operations: Vec<Vec<Operation>>,
    operation_positions: Vec<Vec<usize>>,
    executed_operations: Vec<Vec<Operation>>,
    rng: ThreadRng,
    insert_probability: f32,
    delete_probability: f32,
}

impl FuzzSuite {
    fn new(num_executors: usize, insert_probability: f32, delete_probability: f32) -> FuzzSuite {
        FuzzSuite {
            data_structures: (0..num_executors)
                .map(|id| SynchronizedText::new(id))
                .collect(),
            pushed_operations: vec![vec![]; num_executors],
            executed_operations: vec![vec![]; num_executors],
            operation_positions: vec![vec![0; num_executors]; num_executors],
            rng: thread_rng(),
            insert_probability,
            delete_probability,
        }
    }
    fn execute_all_pending(&mut self) {
        let mut had_pending = true;
        while had_pending {
            had_pending = false;
            let mut execute_succeeded = false;
            for executor in 0..self.num_executors() {
                for from_queue in 0..self.num_executors() {
                    if executor == from_queue {
                        continue;
                    }
                    if !self.has_pending_operations(executor, from_queue) {
                        continue;
                    }
                    had_pending = true;

                    while self.execute_op(executor, from_queue) {
                        execute_succeeded = true;
                    }
                }
            }
            if had_pending && !execute_succeeded {
                panic!("Couldn't execute any operations but had pending ones");
            }
        }
    }

    fn perform_random_operation(&mut self, executor: usize) {
        let random_number: f32 = self.rng.sample(Standard);
        let operation_position = self.data_structures[executor]
            .iter()
            .filter(|(_, c)| c.is_some())
            .map(|(pos, _c)| pos)
            .choose(&mut self.rng);
        let operation_position = operation_position.unwrap_or(S4Vector::root());

        let op =
            if random_number < self.insert_probability || operation_position == S4Vector::root() {
                let inserted = self.rng.sample(Alphanumeric) as char;
                self.data_structures[executor].local_insert(operation_position, inserted)
            } else if random_number < self.insert_probability + self.delete_probability {
                self.data_structures[executor].local_delete(operation_position)
            } else {
                panic!("Update not implemented")
            };
        self.pushed_operations[executor].push(op.clone());
        self.executed_operations[executor].push(op);
    }

    fn num_executors(&self) -> usize {
        self.data_structures.len()
    }
    fn can_execute(&self, executor: usize, from_queue: usize) -> bool {
        executor != from_queue
            && self.has_pending_operations(executor, from_queue)
            && self.is_ready_to_execute(executor, from_queue)
    }

    fn has_pending_operations(&self, executor: usize, from_queue: usize) -> bool {
        self.operation_positions[executor][from_queue] < self.pushed_operations[from_queue].len()
    }

    fn is_ready_to_execute(&self, executor: usize, from_queue: usize) -> bool {
        let op_pos = self.operation_positions[executor][from_queue];

        let op = &self.pushed_operations[from_queue][op_pos];
        self.data_structures[executor].is_ready_to_receive(from_queue, &op.op_clock)
    }

    fn execute_op(&mut self, executor: usize, from_queue: usize) -> bool {
        let op_pos = self.operation_positions[executor][from_queue];
        if !self.can_execute(executor, from_queue) {
            return false;
        }
        let op = &self.pushed_operations[from_queue][op_pos];

        self.data_structures[executor]
            .apply_operation(&op)
            .expect("Failed to apply operation");
        self.executed_operations[executor].push(op.clone());
        self.operation_positions[executor][from_queue] += 1;

        true
    }

    fn has_same_texts(&self) -> bool {
        let text0 = self.data_structures[0].get_text();
        self.data_structures.iter().all(|ds| ds.get_text() == text0)
    }
}

fn op_generation_scheme1(suite: &mut FuzzSuite, num_operations: usize) {
    let mut current_executor = suite.rng.gen_range(0..suite.num_executors());
    let new_executor_probability: f32 = 0.1;
    for _ in 0..num_operations {
        if suite.rng.sample::<f32, _>(Standard) < new_executor_probability {
            current_executor = suite.rng.gen_range(0..suite.num_executors());
        }
        let should_execute = suite.rng.gen_ratio(1, 2);
        if should_execute && apply_random_op(suite, current_executor) {
            continue;
        }
        suite.perform_random_operation(current_executor);
    }
}

fn apply_random_op(suite: &mut FuzzSuite, executor: usize) -> bool {
    let pending: Vec<usize> = (0..suite.num_executors())
        .filter(|idx| suite.can_execute(executor, *idx))
        .collect();
    if pending.is_empty() {
        return false;
    }
    let from_queue = *pending.choose(&mut suite.rng).unwrap();
    suite.execute_op(executor, from_queue);

    true
}

fn print_op_chain(operations: &[Operation]) {
    for op in operations {
        println!("{:?}", op);
    }
}

fn main() {
    let num_ops = 50;
    let num_iterations = 10000;

    for iteration in 0..num_iterations {
        println!("iteration {}", iteration);
        let mut suite = FuzzSuite::new(7, 0.8, 0.2);

        op_generation_scheme1(&mut suite, num_ops);
        suite.execute_all_pending();
        if suite.has_same_texts() {
            continue;
        }

        let text0 = suite.data_structures[0].get_text();
        println!("Original chain");
        print_op_chain(&suite.executed_operations[0]);

        for ds in 0..suite.num_executors() {
            let text = suite.data_structures[ds].get_text();
            if text != text0 {
                println!("expected: {}, received: {}", text0, text);
                print_op_chain(&suite.executed_operations[ds]);
                println!("{:?}", suite.operation_positions[ds]);
            }
        }
        break;
    }
}
