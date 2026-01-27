use soroban_sdk::{symbol_short, Address, BytesN, Env};

// This function publishes an event to the Stellar network
pub fn emit_autoshare_created(env: &Env, id: BytesN<32>, creator: Address) {
    // Topics help indexers filter for this specific event
    let topics = (symbol_short!("created"), creator);

    // Publish the event with the AutoShare ID as the data
    env.events().publish(topics, id);
}

pub fn emit_contract_paused(env: &Env) {
    let topics = (symbol_short!("paused"),);
    env.events().publish(topics, ());
}

pub fn emit_contract_unpaused(env: &Env) {
    let topics = (symbol_short!("unpause"),);
    env.events().publish(topics, ());
}
pub fn emit_autoshare_updated(env: &Env, id: BytesN<32>, updater: Address) {
    let topics = (symbol_short!("updated"), updater);
    env.events().publish(topics, id);
}
