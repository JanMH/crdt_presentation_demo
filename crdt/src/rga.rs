use std::collections::HashMap;

use super::clocks::S4Vector;

pub struct RGA<T> {
    nodes: HashMap<S4Vector, Node<T>>,
}

impl<T: Clone + Default> RGA<T> {
    pub fn new() -> RGA<T> {
        let mut nodes = HashMap::new();
        let n = Node {
            object: T::default(),
            delete_clock: None,
            link: None,
        };
        nodes.insert(S4Vector::root(), n);
        RGA { nodes }
    }
    pub fn insert(&mut self, insert_after: S4Vector, operation_clock: S4Vector, object: T) {
        let mut ref_pos = insert_after;
        while let Some(link) = self.nodes[&ref_pos].link {
            if link < operation_clock {
                break;
            }
            ref_pos = link;
        }

        let reference = self.nodes.get_mut(&ref_pos).unwrap();
        let link = reference.link;
        reference.link = Some(operation_clock);

        self.nodes.insert(
            operation_clock,
            Node {
                object,
                delete_clock: None,
                link,
            },
        );
    }

    pub fn iter(&self) -> SnapshotIter<'_, T> {
        SnapshotIter {
            nodes: &self.nodes,
            link: S4Vector::root(),
        }
    }
}

pub struct SnapshotIter<'a, T> {
    nodes: &'a HashMap<S4Vector, Node<T>>,
    link: S4Vector,
}

impl<T: Clone> Iterator for SnapshotIter<'_, T> {
    type Item = (S4Vector, Option<T>);

    fn next(&mut self) -> Option<Self::Item> {
        let next_link = self.nodes[&self.link].link?;
        self.link = next_link;
        let n = &self.nodes[&self.link];
        let obj = if n.delete_clock.is_none() {
            Some(n.object.clone())
        } else {
            None
        };
        Some((next_link, obj))
    }
}

#[test]
fn test_simple_insertion() {
    use super::clocks::VectorClock;

    let mut rga = RGA::new();
    let mut pos = S4Vector::root();

    let mut clk = VectorClock::new(0);
    clk.increase();
    rga.insert(pos, clk.to_s4vector(), 'h');
    pos = clk.to_s4vector();

    clk.increase();
    rga.insert(pos, clk.to_s4vector(), 'e');
    pos = clk.to_s4vector();

    clk.increase();
    rga.insert(pos, clk.to_s4vector(), 'l');
    pos = clk.to_s4vector();

    clk.increase();
    rga.insert(pos, clk.to_s4vector(), 'l');
    pos = clk.to_s4vector();

    clk.increase();
    rga.insert(pos, clk.to_s4vector(), 'o');

    assert_eq!(
        rga.iter().map(|(_, c)| c.unwrap()).collect::<String>(),
        "hello".to_owned()
    );
}

struct Node<T> {
    object: T,
    delete_clock: Option<S4Vector>,
    link: Option<S4Vector>,
}