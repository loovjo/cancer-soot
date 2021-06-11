mod types;
mod nodes;
mod env;

pub fn example_env() -> (env::LazEnv, nodes::ID) {
    let mut env = env::LazEnv::default();

    let path = nodes::ConstantNode {
        value: types::LazValue::String("src/render/shaders/compiled/data.frag.spv".into()),
    };

    let path_id = env.add_node(Box::new(path));

    let read_file = nodes::ReadFileNode::new(nodes::OutputID { node: path_id, outport: 0 });

    let read_file_id = env.add_node(Box::new(read_file));

    let sum = nodes::SumNode {
        input_list: nodes::OutputID { node: read_file_id, outport: 0 },
    };

    let sum_id = env.add_node(Box::new(sum));

    (env, sum_id)
}
