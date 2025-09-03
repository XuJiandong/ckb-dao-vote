# CKB Dao Vote Specification
This specification describes a type script that implement vote by DAO users on CKB.

## `ckbhash`
CKB uses blake2b as the default hash algorithm. We use `ckbhash` to denote the
blake2b hash function with following configuration:

- output digest size: 32
- personalization: ckb-default-hash

The `blake160` function is defined to return the leading 20 bytes of the `ckbhash` result.

## Vote Meta Cell
The vote meta cell stores metadata for a single vote session. It contains the following cell data in Molecule format:

```text
table VoteMeta {
    smt_root_hash: Byte32Opt,
    candidates: StringVec,
    start_time: Uint64,
    end_time: Uint64,
    extra: BytesOpt,
}
```

To support future extensions, the Molecule format uses the [compatible flag](https://github.com/nervosnetwork/molecule/blob/5d4a3154bc13c8c04b69653f39048fdc2dfd1fb1/bindings/rust/src/prelude.rs#L24), allowing additional fields to be added without breaking compatibility.

### Voter Eligibility (SMT Root Hash)
An off-chain service collects all eligible DAO users and assembles them into a [Sparse Merkle Tree (SMT)](https://github.com/nervosnetwork/sparse-merkle-tree). The SMT structure is as follows:
- **Key**: 32-byte lock script hash of DAO users
- **Value**: Constant 32-byte value `ONE` (`[1,0,0,0,...,0]`)

The SMT root hash is stored in the `smt_root_hash` field:
- **When set**: Only users included in the SMT can vote (restricted vote)
- **When `None`**: All users can vote (open vote)

### Candidates
The `candidates` field contains the vote choices as specified by off-chain services. The type script does not validate the content of these candidates.

### Vote Time Window
The `start_time` and `end_time` fields define the vote period boundaries. Both values are formatted according to the [since](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0017-tx-valid-since/0017-tx-valid-since.md) specification.

Time window validation is performed exclusively by off-chain services. The on-chain type script does not enforce these temporal constraints.

### Extra

The `extra` field is an optional bytes field reserved for off-chain service extensions and metadata. This field allows vote organizers to include additional information.

The on-chain type script does not validate or interpret the contents of this field, ensuring forward compatibility and flexibility for off-chain implementations.


## Type Script Format
The DAO vote type script has the following structure:

```text
Code hash: <DAO vote script code hash>
Hash type: <DAO vote script hash type>
Args:      <blake160 hash of vote meta cell out point, 20 bytes>
```

The `args` field contains the blake160 hash of the vote meta cell's out point. This out point must be present in the transaction's cell dependencies (`cell_deps`), otherwise the script validation will fail.

Once the vote meta cell is consumed in any transaction, the entire vote session is permanently closed and no further votes can be cast. This ensures that each vote session has a definitive end point controlled by the vote organizer.

## Witness Format

Each vote transaction must include a properly formatted `WitnessArgs` data structure in Molecule format. The `output_type` field contains the vote proof with the following structure:

```text
table VoteProof {
    lock_script_hash: Byte32,
    smt_proof: Bytes,
}
```

**Fields:**
- `lock_script_hash`: 32-byte hash of the voter's lock script, used to identify the voter
- `smt_proof`: SMT proof demonstrating the voter's eligibility when SMT validation is enabled. This field is ignored when `smt_root_hash` is `None` in the vote meta cell

## Cell Data Format

The cell data contains exactly one byte representing the voter's choice:

- **Format**: Unsigned 8-bit integer (0-255)
- **Range**: Must correspond to a valid candidate index in the vote meta cell's `candidates` array
- **Validation**: The type script verifies that the choice index does not exceed the number of available candidates


## Validation Procedure

The type script performs the following validation steps in sequence:

**Step 1: Cell Count Analysis**
Initialize `input_count` and `output_count` to zero. The type script iterates through all input cells, incrementing `input_count` when a cell's type script has matching `code_hash` and `hash_type` values. The same process is applied to output cells to determine `output_count`.

**Step 2: Transaction Type Determination**
- If `input_count` is zero and `output_count` is non-zero: Continue validation (vote creation)
- If `input_count` is non-zero and `output_count` is zero: Return success immediately (vote consumption)
- If both `input_count` and `output_count` are non-zero: Validation fails (invalid transaction type)

**Step 3: Vote Meta Cell Verification**
Extract the 20-byte blake160 hash from the current script's args field. Verify that one of the cell dependencies contains an out point hash identical to this value. This cell dependency represents the vote meta cell.

**Step 4: Voter Eligibility Verification**
Read the `smt_root_hash` from the vote meta cell. If the SMT root hash is present (not `None`), use the `lock_script_hash` and `smt_proof` from the corresponding witness to verify that the voter's lock script hash exists in the SMT. This step is skipped when `smt_root_hash` is `None`.

**Step 5: Lock Script Validation**
Verify that at least one input cell contains a lock script whose hash matches the `lock_script_hash` specified in the witness. This ensures the voter controls the claimed identity.

**Step 6: Vote Choice Validation**
Read the first byte of the cell data and convert it to an unsigned integer `index`. Verify that this index is within the valid range (less than the length of the `candidates` array in the vote meta cell).

Steps 4, 5, and 6 are repeated for every cell in the same group of the type script. This allows multiple votes in one transaction.

## Examples

### One Vote in Single Transaction

```
cell_deps:
    <vec> CKB dao vote type script
    <vec> vote meta cell
        smt_root_hash: <SMT root hash>
        candidates: <array of candidates>
        start_time: <not validated by on-chain script>
        end_time: <not validated by on-chain script>
        extra: <not validate by on-chain script>
inputs:
    <vec> cell
        data: <any>
        type: <any>
        lock: <voter's lock script>
outputs:
    <vec> vote cell
        data: <candidate, 1 byte>
        type: <CKB dao vote type script>
            code_hash: <code hash of CKB dao vote type script>
            hash_type: <hash type of CKB dao vote type script>
            args: <blake160 hash of vote meta cell out point, 20 bytes>
        lock: <any>
witnesses:
    <vec> WitnessArgs
      lock:
      input_type:
      output_type: <VoteProof>
        lock_script_hash: <hash of voter's lock script>
        smt_proof: <SMT proof>
```

### Vote Consumption

Release CKB asserts back to users

```
cell_deps:
    <vec> CKB dao vote type script
inputs:
    <vec> cell
        data: <any>
        type: <CKB dao vote type script>
            code_hash: <code hash of CKB dao vote type script>
            hash_type: <hash type of CKB dao vote type script>
            args: <blake160 hash of vote meta cell out point, 20 bytes>
        lock: <lock script>
outputs:
    <vec> any cell

witnesses:
    <vec> WitnessArgs
        lock: <signature to lock script>
        input_type: <none>
        output_type: <none>
```

Note, there is no need to attach vote meta cell in cell_deps.

### Multiple Votes in Single Transaction

```
cell_deps:
    <vec> CKB dao vote type script
    <vec> vote meta cell
        smt_root_hash: <SMT root hash>
        candidates: <array of candidates>
        start_time: <not validated by on-chain script>
        end_time: <not validated by on-chain script>
        extra: <not validated by on-chain script>
inputs:
    <vec> cell
        data: <any>
        type: <any>
        lock: <voter A's lock script>
    <vec> cell
        data: <any>
        type: <any>
        lock: <voter B's lock script>
outputs:
    <vec> vote cell(voter A)
        data: <candidate, 1 byte>
        type: <CKB dao vote type script>
            code_hash: <code hash of CKB dao vote type script>
            hash_type: <hash type of CKB dao vote type script>
            args: <blake160 hash of vote meta cell out point, 20 bytes>
        lock: <any>

    <vec> vote cell(voter B)
        data: <candidate, 1 byte>
        type: <CKB dao vote type script>
            code_hash: <code hash of CKB dao vote type script>
            hash_type: <hash type of CKB dao vote type script>
            args: <blake160 hash of vote meta cell out point, 20 bytes>
        lock: <any>
witnesses:
    <vec> WitnessArgs
      lock:
      input_type:
      output_type: <VoteProof>
        lock_script_hash: <hash of voter A's lock script>
        smt_proof: <SMT proof for voter A>
    <vec> WitnessArgs
      lock:
      input_type:
      output_type: <VoteProof>
        lock_script_hash: <hash of voter B's lock script>
        smt_proof: <SMT proof for voter B>
```


## Deployment

An implementation of the type script spec above has been deployed to CKB mainnet and testnet:

- mainnet

| parameter   | value                                                                |
| ----------- | -------------------------------------------------------------------- |
| `code_hash` | TODO   |
| `hash_type` | `type`                                                               |
| `tx_hash`   | TODO   |
| `index`     | `0x0`                                                                |
| `dep_type`  | `code`                                                               |

- testnet

| parameter   | value                                                                |
| ----------- | -------------------------------------------------------------------- |
| `code_hash` |  TODO |
| `hash_type` | `type`                                                               |
| `tx_hash`   | TODO   |
| `index`     | `0x0`                                                                |
| `dep_type`  | `code`                                                               |

Reproducible build is supported to verify the deployed script. To build the
deployed script above, one can use the following steps:

```bash
TODO
```

