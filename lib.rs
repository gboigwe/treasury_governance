
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod treasury_governance {
    use ink::prelude::string::String;
    use ink::prelude::vec;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    // ========== ENUMS ==========

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ProposalType {
        Treasury,
        Governance,
        Technical,
        Other,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum VotingPeriod {
        ThreeDays,
        SevenDays,
        FourteenDays,
        ThirtyDays,
    }

    impl VotingPeriod {
        /// Convert voting period to block numbers (6 second block time)
        pub fn to_blocks(&self) -> u32 {
            match self {
                VotingPeriod::ThreeDays => 3 * 24 * 60 * 10,      // 43,200 blocks
                VotingPeriod::SevenDays => 7 * 24 * 60 * 10,      // 100,800 blocks
                VotingPeriod::FourteenDays => 14 * 24 * 60 * 10,  // 201,600 blocks
                VotingPeriod::ThirtyDays => 30 * 24 * 60 * 10,    // 432,000 blocks
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum QuorumThreshold {
        Five,
        Ten,
        Twenty,
        TwentyFive,
    }

    impl QuorumThreshold {
        /// Get percentage value
        pub fn to_percentage(&self) -> u32 {
            match self {
                QuorumThreshold::Five => 5,
                QuorumThreshold::Ten => 10,
                QuorumThreshold::Twenty => 20,
                QuorumThreshold::TwentyFive => 25,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ExecutionDelay {
        Immediately,
        OneDay,
        TwoDays,
        SevenDays,
    }

    impl ExecutionDelay {
        /// Convert execution delay to block numbers
        pub fn to_blocks(&self) -> u32 {
            match self {
                ExecutionDelay::Immediately => 0,
                ExecutionDelay::OneDay => 24 * 60 * 10,      // 14,400 blocks
                ExecutionDelay::TwoDays => 2 * 24 * 60 * 10, // 28,800 blocks
                ExecutionDelay::SevenDays => 7 * 24 * 60 * 10, // 100,800 blocks
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ProposalStatus {
        Active,
        Passed,
        Rejected,
        Executed,
        Expired,
    }

    // ========== STRUCTS ==========

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct GovernanceParameters {
        pub voting_period: VotingPeriod,
        pub quorum_threshold: QuorumThreshold,
        pub execution_delay: ExecutionDelay,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct VotingOptions {
        pub options: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct VoteChoice {
        pub option_index: u32,
        pub option_text: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Proposal {
        pub id: u32,
        pub title: String,
        pub description: String,
        pub proposal_type: ProposalType,
        pub governance_params: GovernanceParameters,
        pub voting_options: VotingOptions,
        pub proposer: AccountId,
        pub created_at: u32,
        pub voting_end: u32,
        pub execution_time: u32,
        pub status: ProposalStatus,
        pub vote_counts: Vec<u128>,
        pub total_voters: u32,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Vote {
        pub voter: AccountId,
        pub choice: VoteChoice,
        pub timestamp: u32,
        pub weight: u128,
    }

    // ========== ERROR HANDLING ==========

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        ProposalNotFound,
        ProposalNotActive,
        VotingPeriodEnded,
        AlreadyVoted,
        NotAuthorized,
        ProposalNotReadyForExecution,
        InvalidProposal,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    // ========== EVENTS ==========

    #[ink(event)]
    pub struct ProposalCreated {
        #[ink(topic)]
        proposal_id: u32,
        #[ink(topic)]
        proposer: AccountId,
        title: String,
    }

    #[ink(event)]
    pub struct VoteCast {
        #[ink(topic)]
        proposal_id: u32,
        #[ink(topic)]
        voter: AccountId,
        option_index: u32,
        option_text: String,
        weight: u128,
    }

    #[ink(event)]
    pub struct ProposalExecuted {
        #[ink(topic)]
        proposal_id: u32,
        status: ProposalStatus,
    }

    #[ink(event)]
    pub struct VoterRegistered {
        #[ink(topic)]
        voter: AccountId,
    }

    // ========== STORAGE ==========

    #[ink(storage)]
    pub struct TreasuryGovernance {
        next_proposal_id: u32,
        proposals: Mapping<u32, Proposal>,
        votes: Mapping<(u32, AccountId), Vote>,
        proposal_ids: Vec<u32>,
        total_voters: u32,
        owner: AccountId,
        registered_voters: Mapping<AccountId, bool>,
    }

    // ========== IMPLEMENTATION ==========

    impl TreasuryGovernance {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller_h160 = Self::env().caller();
            // Convert H160 (20 bytes) to AccountId (32 bytes) by padding with zeros
            let mut bytes = [0u8; 32];
            bytes[12..32].copy_from_slice(caller_h160.as_ref());
            let caller = AccountId::from(bytes);

            Self {
                next_proposal_id: 1,
                proposals: Mapping::default(),
                votes: Mapping::default(),
                proposal_ids: Vec::new(),
                total_voters: 0,
                owner: caller,
                registered_voters: Mapping::default(),
            }
        }

        /// Register as a voter
        #[ink(message)]
        pub fn register_voter(&mut self) {
            let caller_h160 = self.env().caller();
            let mut bytes = [0u8; 32];
            bytes[12..32].copy_from_slice(caller_h160.as_ref());
            let caller = AccountId::from(bytes);

            if self.registered_voters.get(caller).is_none() {
                self.registered_voters.insert(caller, &true);
                self.total_voters = self.total_voters.saturating_add(1);

                self.env().emit_event(VoterRegistered { voter: caller });
            }
        }

        /// Create a new proposal
        #[ink(message)]
        pub fn create_proposal(
            &mut self,
            title: String,
            description: String,
            proposal_type: ProposalType,
            governance_params: GovernanceParameters,
            voting_options: VotingOptions,
        ) -> Result<u32> {
            // Validate voting options (1-10 options)
            if voting_options.options.is_empty() || voting_options.options.len() > 10 {
                return Err(Error::InvalidProposal);
            }

            let proposer_h160 = self.env().caller();
            let mut bytes = [0u8; 32];
            bytes[12..32].copy_from_slice(proposer_h160.as_ref());
            let proposer = AccountId::from(bytes);

            let current_block = self.env().block_number();
            let proposal_id = self.next_proposal_id;

            // Calculate voting end time
            let voting_blocks = governance_params.voting_period.to_blocks();
            let voting_end = current_block.saturating_add(voting_blocks);

            // Calculate execution time
            let execution_delay = governance_params.execution_delay.to_blocks();
            let execution_time = voting_end.saturating_add(execution_delay);

            // Initialize vote counts
            let vote_counts = vec![0u128; voting_options.options.len()];

            let proposal = Proposal {
                id: proposal_id,
                title: title.clone(),
                description,
                proposal_type,
                governance_params,
                voting_options,
                proposer,
                created_at: current_block,
                voting_end,
                execution_time,
                status: ProposalStatus::Active,
                vote_counts,
                total_voters: 0,
            };

            self.proposals.insert(proposal_id, &proposal);
            self.proposal_ids.push(proposal_id);
            self.next_proposal_id = self.next_proposal_id.saturating_add(1);

            self.env().emit_event(ProposalCreated {
                proposal_id,
                proposer,
                title,
            });

            Ok(proposal_id)
        }

        /// Vote on a proposal
        #[ink(message)]
        pub fn vote(&mut self, proposal_id: u32, option_index: u32) -> Result<()> {
            let voter_h160 = self.env().caller();
            let mut bytes = [0u8; 32];
            bytes[12..32].copy_from_slice(voter_h160.as_ref());
            let voter = AccountId::from(bytes);

            let current_block = self.env().block_number();

            // Check if proposal exists
            let mut proposal = self.proposals.get(proposal_id)
                .ok_or(Error::ProposalNotFound)?;

            // Check if proposal is active
            if proposal.status != ProposalStatus::Active {
                return Err(Error::ProposalNotActive);
            }

            // Check if voting period has ended
            if current_block > proposal.voting_end {
                return Err(Error::VotingPeriodEnded);
            }

            // Check if already voted
            if self.votes.get((proposal_id, voter)).is_some() {
                return Err(Error::AlreadyVoted);
            }

            // Validate option index
            if option_index as usize >= proposal.voting_options.options.len() {
                return Err(Error::InvalidProposal);
            }

            // Get option text
            let option_text = proposal.voting_options.options[option_index as usize].clone();

            // Create vote with weight of 1
            let vote = Vote {
                voter,
                choice: VoteChoice {
                    option_index,
                    option_text: option_text.clone(),
                },
                timestamp: current_block,
                weight: 1,
            };

            // Update vote counts
            proposal.vote_counts[option_index as usize] = 
                proposal.vote_counts[option_index as usize].saturating_add(1);
            
            proposal.total_voters = proposal.total_voters.saturating_add(1);

            // Store vote and update proposal
            self.votes.insert((proposal_id, voter), &vote);
            self.proposals.insert(proposal_id, &proposal);

            self.env().emit_event(VoteCast {
                proposal_id,
                voter,
                option_index,
                option_text,
                weight: 1,
            });

            Ok(())
        }

        /// Update proposal status after voting period
        #[ink(message)]
        pub fn update_proposal_status(&mut self, proposal_id: u32) -> Result<()> {
            let current_block = self.env().block_number();

            let mut proposal = self.proposals.get(proposal_id)
                .ok_or(Error::ProposalNotFound)?;

            // Only update if active and voting period ended
            if proposal.status != ProposalStatus::Active {
                return Ok(());
            }

            if current_block <= proposal.voting_end {
                return Ok(());
            }

            // Calculate quorum
            let quorum_percentage = proposal.governance_params.quorum_threshold.to_percentage();
            let required_votes = (self.total_voters as u128)
                .saturating_mul(quorum_percentage as u128) / 100;

            let total_votes: u128 = proposal.vote_counts.iter().sum();

            // Check if quorum reached
            if total_votes < required_votes {
                proposal.status = ProposalStatus::Rejected;
                self.proposals.insert(proposal_id, &proposal);
                return Ok(());
            }

            // Find winning option (highest vote count)
            let mut max_votes = 0u128;
            let mut winning_count: u32 = 0;

            for &votes in &proposal.vote_counts {
                if votes > max_votes {
                    max_votes = votes;
                    winning_count = 1;
                } else if votes == max_votes && votes > 0 {
                    winning_count = winning_count.saturating_add(1);
                }
            }

            // Handle ties (mark as rejected)
            if winning_count > 1 {
                proposal.status = ProposalStatus::Rejected;
            } else {
                proposal.status = ProposalStatus::Passed;
            }

            self.proposals.insert(proposal_id, &proposal);
            Ok(())
        }

        /// Execute a passed proposal
        #[ink(message)]
        pub fn execute_proposal(&mut self, proposal_id: u32) -> Result<()> {
            let current_block = self.env().block_number();

            let mut proposal = self.proposals.get(proposal_id)
                .ok_or(Error::ProposalNotFound)?;

            // Check if proposal is in passed status
            if proposal.status != ProposalStatus::Passed {
                return Err(Error::ProposalNotReadyForExecution);
            }

            // Check if execution delay has passed
            if current_block < proposal.execution_time {
                return Err(Error::ProposalNotReadyForExecution);
            }

            proposal.status = ProposalStatus::Executed;
            self.proposals.insert(proposal_id, &proposal);

            self.env().emit_event(ProposalExecuted {
                proposal_id,
                status: ProposalStatus::Executed,
            });

            Ok(())
        }

        // ========== QUERY FUNCTIONS ==========

        /// Get a specific proposal
        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u32) -> Option<Proposal> {
            self.proposals.get(proposal_id)
        }

        /// Get all proposal IDs
        #[ink(message)]
        pub fn get_all_proposal_ids(&self) -> Vec<u32> {
            self.proposal_ids.clone()
        }

        /// Get user's vote on a proposal
        #[ink(message)]
        pub fn get_user_vote(&self, proposal_id: u32, user: AccountId) -> Option<Vote> {
            self.votes.get((proposal_id, user))
        }

        /// Get total registered voters
        #[ink(message)]
        pub fn get_total_voters(&self) -> u32 {
            self.total_voters
        }

        /// Check if proposal reached quorum
        #[ink(message)]
        pub fn has_reached_quorum(&self, proposal_id: u32) -> bool {
            if let Some(proposal) = self.proposals.get(proposal_id) {
                let quorum_percentage = proposal.governance_params.quorum_threshold.to_percentage();
                let required_votes = (self.total_voters as u128)
                    .saturating_mul(quorum_percentage as u128) / 100;
                
                let total_votes: u128 = proposal.vote_counts.iter().sum();
                total_votes >= required_votes
            } else {
                false
            }
        }

        /// Get proposal results
        #[ink(message)]
        pub fn get_proposal_results(&self, proposal_id: u32) -> Option<(Vec<u128>, bool)> {
            if let Some(proposal) = self.proposals.get(proposal_id) {
                let reached_quorum = self.has_reached_quorum(proposal_id);
                Some((proposal.vote_counts, reached_quorum))
            } else {
                None
            }
        }

        /// Get voting options for a proposal
        #[ink(message)]
        pub fn get_voting_options(&self, proposal_id: u32) -> Option<Vec<String>> {
            if let Some(proposal) = self.proposals.get(proposal_id) {
                Some(proposal.voting_options.options)
            } else {
                None
            }
        }

        /// Get detailed results with option names
        #[ink(message)]
        pub fn get_detailed_results(&self, proposal_id: u32) -> Option<Vec<(String, u128)>> {
            if let Some(proposal) = self.proposals.get(proposal_id) {
                let mut results = Vec::new();
                for (i, option) in proposal.voting_options.options.iter().enumerate() {
                    results.push((option.clone(), proposal.vote_counts[i]));
                }
                Some(results)
            } else {
                None
            }
        }

        /// Get the winning option
        #[ink(message)]
        pub fn get_winning_option(&self, proposal_id: u32) -> Option<(String, u128)> {
            if let Some(proposal) = self.proposals.get(proposal_id) {
                let mut max_votes = 0u128;
                let mut winning_index = 0usize;

                for (i, &votes) in proposal.vote_counts.iter().enumerate() {
                    if votes > max_votes {
                        max_votes = votes;
                        winning_index = i;
                    }
                }

                if max_votes > 0 {
                    Some((
                        proposal.voting_options.options[winning_index].clone(),
                        max_votes,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        }

        /// Get contract statistics
        #[ink(message)]
        pub fn get_stats(&self) -> (u32, u32, u32) {
            let total = self.proposal_ids.len() as u32;
            let mut active = 0u32;
            let mut executed = 0u32;

            for &id in &self.proposal_ids {
                if let Some(proposal) = self.proposals.get(id) {
                    match proposal.status {
                        ProposalStatus::Active => active = active.saturating_add(1),
                        ProposalStatus::Executed => executed = executed.saturating_add(1),
                        _ => {}
                    }
                }
            }

            (total, active, executed)
        }
    }

    // ========== TESTS ==========

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Helper function to create default governance parameters
        fn default_governance_params() -> GovernanceParameters {
            GovernanceParameters {
                voting_period: VotingPeriod::ThreeDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            }
        }

        /// Helper function to create default voting options
        fn default_voting_options() -> VotingOptions {
            VotingOptions {
                options: vec![
                    String::from("Approve"),
                    String::from("Reject"),
                    String::from("Abstain"),
                ],
            }
        }

        #[ink::test]
        fn test_1_new_contract_initialization() {
            let contract = TreasuryGovernance::new();
            assert_eq!(contract.get_total_voters(), 0);
            assert_eq!(contract.get_all_proposal_ids().len(), 0);
            let (total, active, executed) = contract.get_stats();
            assert_eq!(total, 0);
            assert_eq!(active, 0);
            assert_eq!(executed, 0);
        }

        #[ink::test]
        fn test_2_voter_registration() {
            let mut contract = TreasuryGovernance::new();
            contract.register_voter();
            assert_eq!(contract.get_total_voters(), 1);
            // Registering again should not increase count
            contract.register_voter();
            assert_eq!(contract.get_total_voters(), 1);
        }

        #[ink::test]
        fn test_3_create_proposal_success() {
            let mut contract = TreasuryGovernance::new();
            let result = contract.create_proposal(
                String::from("Treasury Fund Allocation"),
                String::from("Allocate 1000 tokens for development"),
                ProposalType::Treasury,
                default_governance_params(),
                default_voting_options(),
            );
            assert!(result.is_ok());
            let proposal_id = result.unwrap();
            assert_eq!(proposal_id, 1);

            let proposal = contract.get_proposal(proposal_id).unwrap();
            assert_eq!(proposal.title, String::from("Treasury Fund Allocation"));
            assert_eq!(proposal.status, ProposalStatus::Active);
            assert_eq!(proposal.vote_counts.len(), 3);
        }

        #[ink::test]
        fn test_4_create_proposal_validation() {
            let mut contract = TreasuryGovernance::new();

            // Test empty options
            let empty_options = VotingOptions { options: vec![] };
            let result = contract.create_proposal(
                String::from("Invalid Proposal"),
                String::from("Should fail"),
                ProposalType::Treasury,
                default_governance_params(),
                empty_options,
            );
            assert_eq!(result, Err(Error::InvalidProposal));

            // Test too many options (>10)
            let too_many_options = VotingOptions {
                options: (1..=11).map(|i| format!("Option {}", i)).collect(),
            };
            let result = contract.create_proposal(
                String::from("Invalid Proposal"),
                String::from("Should fail"),
                ProposalType::Treasury,
                default_governance_params(),
                too_many_options,
            );
            assert_eq!(result, Err(Error::InvalidProposal));
        }

        #[ink::test]
        fn test_5_vote_success_and_tracking() {
            let mut contract = TreasuryGovernance::new();
            let proposal_id = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("Test"),
                ProposalType::Treasury,
                default_governance_params(),
                default_voting_options(),
            ).unwrap();

            // Vote on the proposal
            let result = contract.vote(proposal_id, 0);
            assert!(result.is_ok());

            // Verify vote was recorded
            let proposal = contract.get_proposal(proposal_id).unwrap();
            assert_eq!(proposal.vote_counts[0], 1);
            assert_eq!(proposal.total_voters, 1);
        }

        #[ink::test]
        fn test_6_vote_error_cases() {
            let mut contract = TreasuryGovernance::new();
            let proposal_id = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("Test"),
                ProposalType::Treasury,
                default_governance_params(),
                default_voting_options(),
            ).unwrap();

            // Test double voting
            contract.vote(proposal_id, 0).unwrap();
            assert_eq!(contract.vote(proposal_id, 1), Err(Error::AlreadyVoted));

            // Test non-existent proposal
            assert_eq!(contract.vote(999, 0), Err(Error::ProposalNotFound));

            // Test invalid option index
            let proposal_id2 = contract.create_proposal(
                String::from("Test 2"),
                String::from("Test"),
                ProposalType::Treasury,
                default_governance_params(),
                default_voting_options(),
            ).unwrap();
            assert_eq!(contract.vote(proposal_id2, 10), Err(Error::InvalidProposal));
        }

        #[ink::test]
        fn test_7_governance_parameter_enums() {
            // Test voting period conversions (6 second block time)
            assert_eq!(VotingPeriod::ThreeDays.to_blocks(), 43_200);
            assert_eq!(VotingPeriod::SevenDays.to_blocks(), 100_800);
            assert_eq!(VotingPeriod::FourteenDays.to_blocks(), 201_600);
            assert_eq!(VotingPeriod::ThirtyDays.to_blocks(), 432_000);

            // Test quorum threshold percentages
            assert_eq!(QuorumThreshold::Five.to_percentage(), 5);
            assert_eq!(QuorumThreshold::Ten.to_percentage(), 10);
            assert_eq!(QuorumThreshold::Twenty.to_percentage(), 20);
            assert_eq!(QuorumThreshold::TwentyFive.to_percentage(), 25);

            // Test execution delays
            assert_eq!(ExecutionDelay::Immediately.to_blocks(), 0);
            assert_eq!(ExecutionDelay::OneDay.to_blocks(), 14_400);
            assert_eq!(ExecutionDelay::TwoDays.to_blocks(), 28_800);
            assert_eq!(ExecutionDelay::SevenDays.to_blocks(), 100_800);
        }

        #[ink::test]
        fn test_8_quorum_calculation() {
            let mut contract = TreasuryGovernance::new();
            contract.total_voters = 10; // Simulate 10 registered voters

            let proposal_id = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("Test"),
                ProposalType::Treasury,
                default_governance_params(), // 10% quorum
                default_voting_options(),
            ).unwrap();

            // With 10 total voters and 10% quorum, need 1 vote minimum
            assert_eq!(contract.has_reached_quorum(proposal_id), false);

            contract.vote(proposal_id, 0).unwrap();
            assert_eq!(contract.has_reached_quorum(proposal_id), true);
        }

        #[ink::test]
        fn test_9_query_functions() {
            let mut contract = TreasuryGovernance::new();
            let proposal_id = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("Test Description"),
                ProposalType::Treasury,
                default_governance_params(),
                default_voting_options(),
            ).unwrap();

            contract.vote(proposal_id, 0).unwrap();

            // Test get_voting_options
            let options = contract.get_voting_options(proposal_id).unwrap();
            assert_eq!(options.len(), 3);
            assert_eq!(options[0], String::from("Approve"));

            // Test get_detailed_results
            let results = contract.get_detailed_results(proposal_id).unwrap();
            assert_eq!(results[0], (String::from("Approve"), 1));
            assert_eq!(results[1], (String::from("Reject"), 0));

            // Test get_winning_option
            let (winner, votes) = contract.get_winning_option(proposal_id).unwrap();
            assert_eq!(winner, String::from("Approve"));
            assert_eq!(votes, 1);

            // Test get_proposal_results
            let (vote_counts, _) = contract.get_proposal_results(proposal_id).unwrap();
            assert_eq!(vote_counts[0], 1);
        }

        #[ink::test]
        fn test_10_multiple_proposals_management() {
            let mut contract = TreasuryGovernance::new();

            let id1 = contract.create_proposal(
                String::from("Proposal 1"),
                String::from("First"),
                ProposalType::Treasury,
                default_governance_params(),
                default_voting_options(),
            ).unwrap();

            let id2 = contract.create_proposal(
                String::from("Proposal 2"),
                String::from("Second"),
                ProposalType::Governance,
                default_governance_params(),
                default_voting_options(),
            ).unwrap();

            assert_eq!(id1, 1);
            assert_eq!(id2, 2);

            let all_ids = contract.get_all_proposal_ids();
            assert_eq!(all_ids.len(), 2);

            let (total, active, _) = contract.get_stats();
            assert_eq!(total, 2);
            assert_eq!(active, 2);
        }

        #[ink::test]
        fn test_11_custom_voting_options() {
            let mut contract = TreasuryGovernance::new();

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
                String::from("Custom Options Test"),
                String::from("Testing 5-point scale"),
                ProposalType::Governance,
                default_governance_params(),
                custom_options,
            ).unwrap();

            let options = contract.get_voting_options(proposal_id).unwrap();
            assert_eq!(options.len(), 5);
            assert_eq!(options[0], String::from("Strongly Agree"));
            assert_eq!(options[4], String::from("Strongly Disagree"));
        }

        #[ink::test]
        fn test_12_proposal_types_and_execution() {
            let mut contract = TreasuryGovernance::new();

            // Test different proposal types
            let treasury_id = contract.create_proposal(
                String::from("Treasury Proposal"),
                String::from("Fund allocation"),
                ProposalType::Treasury,
                default_governance_params(),
                default_voting_options(),
            ).unwrap();

            let governance_id = contract.create_proposal(
                String::from("Governance Change"),
                String::from("Protocol update"),
                ProposalType::Governance,
                default_governance_params(),
                default_voting_options(),
            ).unwrap();

            let technical_id = contract.create_proposal(
                String::from("Technical Upgrade"),
                String::from("System enhancement"),
                ProposalType::Technical,
                default_governance_params(),
                default_voting_options(),
            ).unwrap();

            assert_eq!(contract.get_proposal(treasury_id).unwrap().proposal_type, ProposalType::Treasury);
            assert_eq!(contract.get_proposal(governance_id).unwrap().proposal_type, ProposalType::Governance);
            assert_eq!(contract.get_proposal(technical_id).unwrap().proposal_type, ProposalType::Technical);

            // Test execution validation
            let result = contract.execute_proposal(treasury_id);
            assert_eq!(result, Err(Error::ProposalNotReadyForExecution));
        }
    }
}
