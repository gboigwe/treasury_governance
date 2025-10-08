# Treasury Governance Smart Contract

A treasury governance smart contract built with ink! (Rust-based smart contract framework) for Polkadot/Substrate. This contract enables decentralized decision-making for treasury fund allocation through proposal creation, voting, and execution mechanisms.

## 🎯 Features

- **Proposal Management**: Create and manage governance proposals with customizable parameters
- **Flexible Voting System**: Support for custom voting options (not just Yes/No)
- **Quorum Requirements**: Configurable quorum thresholds (5%, 10%, 20%, 25%)
- **Voting Periods**: Multiple voting period options (3, 7, 14, 30 days)
- **Execution Delays**: Safety delays before proposal execution
- **Proposal Types**: Treasury, Governance, Technical, and Other proposals
- **Voter Registration**: Track registered voters for quorum calculations
- **Comprehensive Queries**: Rich set of query functions for proposal data

## 📋 Prerequisites

- Rust (latest stable version)
- `cargo-contract` CLI tool for ink! smart contracts

### Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-contract
cargo install cargo-contract --force
```

## 🚀 Quick Start

### Build the Contract

```bash
cargo contract build
```

This generates the following artifacts in `target/ink/`:
- `treasury.contract` - Contract bundle (code + metadata)
- `treasury.polkavm` - Contract bytecode
- `treasury.json` - Contract metadata

### Run Tests

```bash
cargo test
```

Expected output:
```
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored
```

## 📖 Contract Architecture

### Core Data Structures

#### Proposal Types
```rust
pub enum ProposalType {
    Treasury,    // Fund allocation proposals
    Governance,  // Protocol change proposals
    Technical,   // Technical improvement proposals
    Other,       // Miscellaneous proposals
}
```

#### Governance Parameters
```rust
pub struct GovernanceParameters {
    pub voting_period: VotingPeriod,        // 3, 7, 14, or 30 days
    pub quorum_threshold: QuorumThreshold,  // 5%, 10%, 20%, or 25%
    pub execution_delay: ExecutionDelay,    // Immediate, 1, 2, or 7 days
}
```

#### Proposal Status
```rust
pub enum ProposalStatus {
    Active,    // Currently accepting votes
    Passed,    // Voting ended, quorum reached, ready for execution
    Rejected,  // Voting ended, did not meet quorum or tied
    Executed,  // Proposal has been executed
    Expired,   // Voting period expired
}
```

### Main Contract Functions

#### Constructor
```rust
#[ink(constructor)]
pub fn new() -> Self
```
Initialize the contract with default values.

#### Voter Registration
```rust
#[ink(message)]
pub fn register_voter(&mut self)
```
Register as a voter (required for quorum calculations).

#### Create Proposal
```rust
#[ink(message)]
pub fn create_proposal(
    &mut self,
    title: String,
    description: String,
    proposal_type: ProposalType,
    governance_params: GovernanceParameters,
    voting_options: VotingOptions,
) -> Result<u32>
```
Create a new proposal with custom voting options (1-10 options).

#### Vote
```rust
#[ink(message)]
pub fn vote(&mut self, proposal_id: u32, option_index: u32) -> Result<()>
```
Cast a vote on an active proposal. Each account can only vote once per proposal.

#### Update Proposal Status
```rust
#[ink(message)]
pub fn update_proposal_status(&mut self, proposal_id: u32) -> Result<()>
```
Update proposal status after voting period ends (checks quorum, determines winner).

#### Execute Proposal
```rust
#[ink(message)]
pub fn execute_proposal(&mut self, proposal_id: u32) -> Result<()>
```
Execute a passed proposal after the execution delay.

### Query Functions

```rust
// Get specific proposal
pub fn get_proposal(&self, proposal_id: u32) -> Option<Proposal>

// Get all proposal IDs
pub fn get_all_proposal_ids(&self) -> Vec<u32>

// Get user's vote on a proposal
pub fn get_user_vote(&self, proposal_id: u32, user: AccountId) -> Option<Vote>

// Get total registered voters
pub fn get_total_voters(&self) -> u32

// Check if proposal reached quorum
pub fn has_reached_quorum(&self, proposal_id: u32) -> bool

// Get proposal results
pub fn get_proposal_results(&self, proposal_id: u32) -> Option<(Vec<u128>, bool)>

// Get voting options
pub fn get_voting_options(&self, proposal_id: u32) -> Option<Vec<String>>

// Get detailed results with option names
pub fn get_detailed_results(&self, proposal_id: u32) -> Option<Vec<(String, u128)>>

// Get winning option
pub fn get_winning_option(&self, proposal_id: u32) -> Option<(String, u128)>

// Get contract statistics
pub fn get_stats(&self) -> (u32, u32, u32) // (total, active, executed)
```

## 💡 Usage Examples

### Example 1: Create a Treasury Proposal

```rust
let governance_params = GovernanceParameters {
    voting_period: VotingPeriod::SevenDays,
    quorum_threshold: QuorumThreshold::Twenty,
    execution_delay: ExecutionDelay::TwoDays,
};

let voting_options = VotingOptions {
    options: vec![
        String::from("Approve Full Amount"),
        String::from("Approve Half Amount"),
        String::from("Reject"),
    ],
};

let proposal_id = contract.create_proposal(
    String::from("Community Development Fund"),
    String::from("Allocate 10,000 tokens for community development initiatives"),
    ProposalType::Treasury,
    governance_params,
    voting_options,
)?;
```

### Example 2: Vote on a Proposal

```rust
// Register as voter (one-time)
contract.register_voter();

// Vote on proposal (option index 0 = "Approve Full Amount")
contract.vote(proposal_id, 0)?;
```

### Example 3: Custom Voting Options

```rust
let custom_options = VotingOptions {
    options: vec![
        String::from("Strongly Agree"),
        String::from("Agree"),
        String::from("Neutral"),
        String::from("Disagree"),
        String::from("Strongly Disagree"),
    ],
};

let proposal_id = contract.create_proposal(
    String::from("Protocol Upgrade Vote"),
    String::from("Should we upgrade to version 2.0?"),
    ProposalType::Governance,
    governance_params,
    custom_options,
)?;
```

## 🧪 Testing

The contract includes 12 comprehensive tests covering:

1. **Contract Initialization** - Verify default state
2. **Voter Registration** - Test voter registration and duplicate prevention
3. **Proposal Creation** - Test successful proposal creation
4. **Proposal Validation** - Test input validation (empty/too many options)
5. **Vote Success** - Test successful voting and tracking
6. **Vote Error Cases** - Test double voting, invalid proposals, invalid options
7. **Governance Parameters** - Test enum conversions (blocks, percentages)
8. **Quorum Calculation** - Test quorum logic
9. **Query Functions** - Test all query functions
10. **Multiple Proposals** - Test managing multiple proposals
11. **Custom Voting Options** - Test flexible voting options
12. **Proposal Types** - Test different proposal types and execution

Run tests with:
```bash
cargo test
```

## 📊 Contract Statistics

After deployment, you can query contract statistics:

```rust
let (total_proposals, active_proposals, executed_proposals) = contract.get_stats();
```

## ⚙️ Configuration

### Block Time Assumptions

The contract assumes a 6-second block time for time calculations:
- 1 minute = 10 blocks
- 1 hour = 600 blocks
- 1 day = 14,400 blocks

### Voting Options Limits

- Minimum: 1 option
- Maximum: 10 options

### Proposal Types

Choose the appropriate type for your proposal:
- **Treasury**: For fund allocation and budget proposals
- **Governance**: For protocol changes and governance updates
- **Technical**: For technical improvements and upgrades
- **Other**: For miscellaneous proposals

## 🔐 Security Considerations

1. **Double Voting Prevention**: Each account can only vote once per proposal
2. **Quorum Requirements**: Proposals must meet minimum participation thresholds
3. **Execution Delays**: Safety delays prevent immediate execution of passed proposals
4. **Input Validation**: All inputs are validated (voting options, proposal IDs, etc.)

## 🛠️ Development

### Project Structure

```
treasury/
├── Cargo.toml          # Project configuration
├── lib.rs              # Contract implementation
├── README.md           # This file
└── target/
    └── ink/            # Build artifacts
        ├── treasury.contract
        ├── treasury.polkavm
        └── treasury.json
```

### Building in Release Mode

```bash
cargo contract build --release
```

### Checking Code

```bash
cargo check
cargo clippy
```

## 📝 Error Handling

The contract uses a custom `Error` enum for error handling:

```rust
pub enum Error {
    ProposalNotFound,              // Proposal ID doesn't exist
    ProposalNotActive,             // Proposal is not in Active status
    VotingPeriodEnded,             // Voting period has ended
    AlreadyVoted,                  // User has already voted
    NotAuthorized,                 // User not authorized
    ProposalNotReadyForExecution,  // Proposal not ready to execute
    InvalidProposal,               // Invalid proposal parameters
}
```

## 🤝 Contributing

This contract was built as part of a Polkadot/Substrate workshop. Contributions and improvements are welcome!

## 📄 License

This project is open source and available for educational and development purposes.

## 🔗 Resources

- [ink! Documentation](https://use.ink/)
- [Substrate Documentation](https://docs.substrate.io/)
- [Polkadot Documentation](https://polkadot.network/docs/)
- [Rust Documentation](https://doc.rust-lang.org/)

## 📧 Support

For issues, questions, or contributions, please refer to the project repository.

---

**Built with ❤️ using ink! for Polkadot/Substrate**
