mod reset;
mod add;

use serenity::all::{CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage, Permissions};
use serenity::builder::CreateCommand;

pub fn register() -> CreateCommand {
    CreateCommand::new("creator-channel")
        .description("Adds a creator channel")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(
            add::get_command_option()
        )
        .add_option(
            reset::get_command_option()
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> CreateInteractionResponse {
    let option = match command.data.options.first() {
        None => return create_response("Unknown subcommand!"),
        Some(option) => option,
    };

    match option.name.as_str() {
        "add" => add::run(ctx, command).await,
        "reset" => reset::run(ctx, command).await,
        _ => create_response("Unknown subcommand!"),
    }
}

fn create_response(string: &str) -> CreateInteractionResponse {
    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(string)
    )
}