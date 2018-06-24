extern crate timely;

use std::collections::HashMap;

use timely::dataflow::InputHandle;
use timely::dataflow::operators::{Input, Inspect, Probe};
use timely::dataflow::operators::generic::operator::Operator;
use timely::dataflow::channels::pact::Exchange;

fn main() {
    // initializes and runs a timely dataflow.
    timely::execute_from_args(std::env::args(), |worker| {
        let index = worker.index();
        let mut input = InputHandle::new();

        // create a new input, exchange data, and inspect its output
        let probe = worker.dataflow(|scope| {
            let mut counts_by_time = HashMap::new();
            scope.input_from(&mut input)
                .unary(Exchange::new(|x| *x), "Distinct", move |_, _|
                    move |input, output| {
                        input.for_each(|time, data| {
                            let counts = counts_by_time.entry(time.time().clone())
                                .or_insert(HashMap::new());
                            let mut session = output.session(&time);
                            for &datum in data.iter() {
                                let count = counts.entry(datum).or_insert(0);
                                if *count == 0 {
                                    session.give(datum);
                                }
                                *count += 1;
                            }
                        })
                    })
                .inspect(move |x| println!("worker {}:\tvalue {}", index, x))
                .probe()
        });

        // introduce data and watch!
        for round in 0..1 {
            if index == 0 {
                vec![0, 1, 2, 2, 2, 3, 3, 4].iter().for_each(|x| input.send(*x));
            } else if index == 1 {
                vec![0, 0, 3, 4, 4, 5, 7, 7].iter().for_each(|x| input.send(*x));
            }
            input.advance_to(round + 1);
            while probe.less_than(input.time()) {
                worker.step();
            }
        }
    }).unwrap();
}
