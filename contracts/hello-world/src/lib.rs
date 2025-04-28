#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, String, symbol_short, Symbol};

// Define health record structure
#[contracttype]
#[derive(Clone)]
pub struct HealthRecord {
    pub user_id: u64,
    pub health_score: u32,
    pub last_updated: u64,
    pub goals_achieved: u32,
    pub active_goals: u32,
}

// Define goal structure
#[contracttype]
#[derive(Clone)]
pub struct HealthGoal {
    pub goal_id: u64,
    pub user_id: u64,
    pub description: String,
    pub target_value: u32,
    pub current_value: u32,
    pub deadline: u64,
    pub completed: bool,
}

// Keys for storage
const USER_COUNT: Symbol = symbol_short!("USER_CNT");
const GOAL_COUNT: Symbol = symbol_short!("GOAL_CNT");

// Contract declaration
#[contract]
pub struct HealthCoachContract;

#[contractimpl]
impl HealthCoachContract {
    // Create or update a user's health record
    pub fn update_health_record(
        env: Env, 
        user_id: u64, 
        health_score: u32, 
        goals_achieved: u32, 
        active_goals: u32
    ) -> HealthRecord {
        let time = env.ledger().timestamp();
        
        let record = HealthRecord {
            user_id,
            health_score,
            last_updated: time,
            goals_achieved,
            active_goals,
        };
        
        // Store the updated health record
        env.storage().instance().set(&user_id, &record);
        
        // If this is a new user, increment the user count
        if !env.storage().instance().has(&user_id) {
            let mut count: u64 = env.storage().instance().get(&USER_COUNT).unwrap_or(0);
            count += 1;
            env.storage().instance().set(&USER_COUNT, &count);
        }
        
        env.storage().instance().extend_ttl(1000, 1000);
        record
    }
    
    // Create a new health goal for a user
    pub fn create_health_goal(
        env: Env,
        user_id: u64,
        description: String,
        target_value: u32,
        deadline: u64
    ) -> HealthGoal {
        // Get the next goal ID
        let mut goal_count: u64 = env.storage().instance().get(&GOAL_COUNT).unwrap_or(0);
        goal_count += 1;
        
        let goal = HealthGoal {
            goal_id: goal_count,
            user_id,
            description,
            target_value,
            current_value: 0,
            deadline,
            completed: false,
        };
        
        // Store the goal
        env.storage().instance().set(&goal_count, &goal);
        
        // Update the goal count
        env.storage().instance().set(&GOAL_COUNT, &goal_count);
        
        // Update the user's active goals count
        if let Some(mut record) = env.storage().instance().get::<u64, HealthRecord>(&user_id) {
            record.active_goals += 1;
            env.storage().instance().set(&user_id, &record);
        }
        
        env.storage().instance().extend_ttl(1000, 1000);
        goal
    }
    
    // Update progress on a health goal
    pub fn update_goal_progress(
        env: Env,
        goal_id: u64,
        current_value: u32
    ) -> HealthGoal {
        // Get the goal
        let mut goal: HealthGoal = env.storage().instance().get(&goal_id).unwrap_or_else(|| {
            panic!("Goal not found");
        });
        
        // Update the current value
        goal.current_value = current_value;
        
        // Check if goal is now completed
        if current_value >= goal.target_value {
            goal.completed = true;
            
            // Update the user's record
            if let Some(mut record) = env.storage().instance().get::<u64, HealthRecord>(&goal.user_id) {
                record.goals_achieved += 1;
                record.active_goals -= 1;
                env.storage().instance().set(&goal.user_id, &record);
            }
        }
        
        // Store the updated goal
        env.storage().instance().set(&goal_id, &goal);
        
        env.storage().instance().extend_ttl(1000, 1000);
        goal
    }
    
    // Get a user's health record
    pub fn get_health_record(env: Env, user_id: u64) -> HealthRecord {
        env.storage().instance().get(&user_id).unwrap_or_else(|| {
            panic!("User not found");
        })
    }
}