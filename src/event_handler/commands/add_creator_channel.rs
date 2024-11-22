use std::collections::HashMap;

use serenity::all::{ChannelId, CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType, Context, CreateInteractionResponse, CreateInteractionResponseMessage, Permissions};
use serenity::builder::{CreateCommand, CreateCommandOption};

use crate::creator_channel::CreatorChannelConfig;
use crate::StorageKey;

pub fn register() -> CreateCommand {
    CreateCommand::new("add-creator-channel")
        .description("Adds a creator channel")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(
            CreateCommandOption::new(CommandOptionType::Channel, "creator_id", "Channel to be the creator channel")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::Channel, "category_id", "Category for the temporary channel to be created in")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "naming_standard", "Naming standard")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "user_limit", "User limit")
                .required(true),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> CreateInteractionResponse {
    let creator_channel_config = match get_creator_channel_config(command) {
        None => return create_response("Something went wrong when trying to parse the command options!"),
        Some(creator_channel_config) => creator_channel_config,
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

    storage.set_creator_voice_config(&creator_channel_config).await;

    return create_response("Added creator channel to the database!");
}

macro_rules! extract_option {
    ($map:expr, $key:expr, $method:ident) => {
        $map.get($key)?.$method()
    };
}

fn get_creator_channel_config(command: &CommandInteraction) -> Option<CreatorChannelConfig> {
    let guild_id = match command.guild_id {
        None => return None,
        Some(guild_id) =>guild_id,
    };

    let options = &command.data.options;

    // HashMap<&str, &CommandDataOption>

    let option_map: HashMap<&str, &CommandDataOptionValue> = HashMap::from_iter(
        options
            .iter()
            .map(|opt| (opt.name.as_str(), &opt.value))
    );

    let creator_id:  ChannelId  = extract_option!(option_map, "creator_id",      as_channel_id)?;
    let category_id: ChannelId  = extract_option!(option_map, "category_id",     as_channel_id)?;
    let naming_standard: String = extract_option!(option_map, "naming_standard", as_str)?.to_string();
    let user_limit: u32         = extract_option!(option_map, "user_limit",      as_i64)? as u32;

    return Some(
        CreatorChannelConfig{
            guild_id,
            creator_id,
            category_id,
            naming_standard,
            channel_numbers: Default::default(),
            user_limit,
        }
    )
}

fn create_response(string: &str) -> CreateInteractionResponse {
    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(string)
    )
}