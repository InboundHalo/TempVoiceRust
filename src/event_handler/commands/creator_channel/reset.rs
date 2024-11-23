use std::collections::HashMap;

use serenity::all::{CommandDataOptionValue, CommandInteraction, CommandOptionType, Context, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::builder::CreateCommandOption;

use crate::StorageKey;

macro_rules! extract_option {
    ($map:expr, $key:expr, $method:ident) => {
        match $map.get($key) {
            None => None,
            Some(command_data_option_value) => match command_data_option_value.$method() {
                None => None,
                Some(value) => Some(value),
            }
        }
    };
}

pub fn get_command_option() -> CreateCommandOption {
    CreateCommandOption::new(CommandOptionType::SubCommand, "reset", "Resets a creator channel")
        .add_sub_option(
            CreateCommandOption::new(CommandOptionType::Channel, "creator_id", "Channel to be reset")
                .required(true),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> CreateInteractionResponse {
    println!("Running creator-channel reset");

    let reset_option = match command.data.options.iter().find(|opt| opt.name == "reset") {
        None => return create_response("Something went wrong when trying to parse the command options!"),
        Some(command_data_option) => command_data_option,
    };

    let sub_options = match &reset_option.value {
        CommandDataOptionValue::SubCommand(options) => options,
        CommandDataOptionValue::SubCommandGroup(options) => options,
        _ => return create_response("Invalid subcommand or subcommand group format!"),
    };

    let option_map: HashMap<&str, &CommandDataOptionValue> = HashMap::from_iter(
        sub_options.iter().map(|opt| (opt.name.as_str(), &opt.value))
    );
    let creator_id = match extract_option!(option_map, "creator_id", as_channel_id) {
        None => return create_response("Something went wrong when trying to parse the command options!"),
        Some(channel_id) => channel_id,
    };

    let storage = {
        let data_read = ctx.data.read().await;
        match data_read.get::<StorageKey>().cloned() {
            None => {
                println!("Storage is null!");
                panic!()
            }
            Some(storage) => storage,
        }
    };

    let mut creator_channel = match storage.get_creator_voice_config(&creator_id).await {
        None => return create_response("That channel is not a creator channel!"),
        Some(creator_channel) => creator_channel,
    };

    creator_channel.reset();

    storage.set_creator_voice_config(&creator_channel).await;



    create_response("Reset completed successfully!")
}

fn create_response(string: &str) -> CreateInteractionResponse {
    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(string)
    )
}