

## Components

- `in_array(a[], x)`: Check if `Prod_i (x-a[i])` is 0

- `query(k[], v[], x)`: Assume the keys in key are all distinct.
  If not in_array(k, x) -> 0. Else  `inner_product(is_zero(k[i] - x), v)`

- `query_sum(k[], v[], x)`: The same as above, but when there can be duplicate keys,
  the result is the sum of all values.


## Solutions

### V1: O(`MAX_OPS * MAX_MEM`)

The most naive approach is to keep a copy of memory at every step.
At each step query memory at a certain location can be done by `select_from_idx`

### V2: assuming at most one update at each position
Keep an update_key and update_value arrays, both with size MAX_OPS.
At time t, query the current value at position x by `query` above.
If op type is write, we put in the update_key and update_value the ops_ptr, ops_value.
If op type is read, we put in some dummy value.

### V3: O(`MAX_MEM + MAX_OPS^2`)
Very similar to above, if we use `query_sum` now we can allow multiple keys, and thus updating new values.
The new entry of update_value is `ops_value - current_value`

### V4 Hybrid: O(`MAX_MEM*MAX_OPS/r + r * MAX_OPS`)
Keep a cache value of memory state every r steps.

## Implementation
The code is V3

### Caveats
Due to time constraint I am leaving some further improvements
- Currently assume all ops are of type 1 or 2. TODO: implement ignoring op 0
- Currently assume the MAX_OPS is equal to num_ops. Related to above, can just use op=0 to pad

### data
```
RUST_BACKTRACE=full cargo +nightly run -- -k 12 -i data/input1.json  -p https://sepolia.infura.io/v3/XX  keygen/prove
```

Good examples:
- input: dummy example
- input1: write and read should get new value
- input2: write to multiple location
- input3: update same location multiple times

Bad examples:
- input4: read not getting the latest value
