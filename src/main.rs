use std::{char::MAX, collections::HashMap, fmt::Debug};

use axiom_sdk::{
    axiom::{AxiomAPI, AxiomComputeFn, AxiomComputeInput, AxiomResult},
    axiom_circuit::{self, axiom_eth::snark_verifier::loader::halo2::IntegerInstructions, input::flatten::FixLenVec},
    cmd::run_cli,
    ethers::core::k256::elliptic_curve::PrimeField,
    halo2_base::{
        gates::{GateChip, GateInstructions, RangeInstructions},
        AssignedValue,
        QuantumCell,
    },
    Fr
};

const MAX_OPS: usize = 10;
const MAX_MEM: usize = 1024;

#[AxiomComputeInput]
pub struct OpsInput {
    pub ops_flag: FixLenVec<usize, MAX_OPS>,
    pub ops_ptr: FixLenVec<usize, MAX_OPS>,
    pub ops_value: FixLenVec<usize, MAX_OPS>,
    pub num_ops: usize,
}

fn in_vec<Q>(
    api: &mut AxiomAPI,
    x: AssignedValue<Fr>,
    a: impl IntoIterator<Item = Q>
) -> AssignedValue<Fr> 
where
    Q: Into<QuantumCell<Fr>>,
{
    let gate = api.range.gate();
    let a = a.into_iter();
    let (len, hi) = a.size_hint();
    let mut prod = Fr::one();
    a.map(|y| {
        let y: QuantumCell<Fr> = y.into();
        prod = prod * (*y.value()-*x.value());
    });
    let result = api.ctx().load_witness(prod);
    gate.is_zero(api.ctx(), result)
}

fn query_sum<Q>(
    api: &mut AxiomAPI,
    x: AssignedValue<Fr>,
    ks: impl IntoIterator<Item = Q>,
    vs: impl IntoIterator<Item = QuantumCell<Fr>>,
) -> AssignedValue<Fr> 
where
    Q: Into<QuantumCell<Fr>>,
{
    let gate = api.range.gate();
    let ctx = api.ctx();
    let ks = ks.into_iter();
    let (len, hi) = ks.size_hint();
    let ind: Vec<QuantumCell<Fr>> = ks.map(|k| {
        let k: QuantumCell<Fr> = k.into();
        let diff = GateInstructions::sub(gate,ctx, k, x);
        QuantumCell::Existing(gate.is_zero(ctx, diff))
    }).collect();
    gate.inner_product(ctx,  vs.into_iter(), ind)
}

// use two less than as i don't find an assert equal constrain
fn assert_equal(
    api: &mut AxiomAPI,
    a: AssignedValue<Fr>, 
    b: AssignedValue<Fr>, 
) {
    let gate = api.range.gate();
    let a_plus1 = gate.inc(api.ctx(), QuantumCell::Existing(a));
    api.range.check_less_than(api.ctx(), QuantumCell::Existing(b), QuantumCell::Existing(a_plus1), 12);
    let b_plus1 = gate.inc(api.ctx(), QuantumCell::Existing(b));
    api.range.check_less_than(api.ctx(), QuantumCell::Existing(a), QuantumCell::Existing(b_plus1), 12);
}

impl AxiomComputeFn for OpsInput {

    fn compute(
        api: &mut AxiomAPI,
        assigned_inputs: OpsCircuitInput<AssignedValue<Fr>>,
    ) -> Vec<AxiomResult> {
        let gate = api.range.gate();
        let zero = api.ctx().load_zero();
        let one = api.ctx().load_constant(Fr::one());

        let max_ops_plus_1 = api.ctx().load_constant(Fr::from_u128((MAX_OPS + 1) as u128));
        api.range.check_less_than(api.ctx(), assigned_inputs.num_ops, max_ops_plus_1, 12);

        let mut update_keys: Vec<AssignedValue<Fr>> = Vec::new();
        let mut update_vals: Vec<AssignedValue<Fr>> = Vec::new();
        // TODO: I am changing max_ops to equal to num ops when running for now.
        for t in 0..MAX_OPS {
            let ptr = *assigned_inputs.ops_ptr.get(t).unwrap();
            let value = *assigned_inputs.ops_value.get(t).unwrap();
            let ks = update_keys.iter().take(t).map(|k| {
                QuantumCell::Existing(k.clone())
            });
            let vs = update_vals.iter().take(t).map(|v| {
                QuantumCell::Existing(v.clone())
            });
            let cur_val = query_sum(api, ptr, ks, vs);
            let op_type = *assigned_inputs.ops_flag.get(t).unwrap();

            // TODO: assume op is 1 or 2 (no 0)
            let is_read = gate.is_equal(api.ctx(), op_type, one);

            // CONSTRAINT: dummy constraint for write
            let target_val = gate.select(api.ctx(), cur_val, value, is_read);
            assert_equal(api, value, target_val);

            // UPDATE
            let diff = GateInstructions::sub(gate, api.ctx(), value, cur_val);
            // if it's read, update a dummy thing: max_ops_plus_1 -> 0
            let new_update_k = gate.select(api.ctx(), max_ops_plus_1, ptr, is_read);
            let new_update_v = gate.select(api.ctx(), zero, diff, is_read);
            update_keys.push(new_update_k);
            update_vals.push(new_update_v);

        }
        
        vec![zero.into()]
        
    }
}

fn main() {
    //env_logger::init();
    run_cli::<OpsInput>();
}