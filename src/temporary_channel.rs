use std::num::NonZeroU16;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serenity::all::{ActivityType, ChannelId, Context, GuildId, Presence, UserId};
use serenity::futures::AsyncReadExt;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemporaryVoiceChannel {
    pub(crate) guild_id: GuildId,
    pub(crate) channel_id: ChannelId,
    pub(crate) creator_id: ChannelId,
    pub(crate) owner_id: UserId,
    pub(crate) name: String,
    pub(crate) template_name: String,
    pub(crate) number: NonZeroU16,
}

impl TemporaryVoiceChannel {
    pub fn new(
        guild_id: GuildId,
        channel_id: ChannelId,
        creator_id: ChannelId,
        owner_id: UserId,
        name: String,
        template_name: String,
        number: NonZeroU16,
    ) -> Self {
        Self {
            guild_id,
            channel_id,
            creator_id,
            owner_id,
            name,
            template_name,
            number,
        }
    }
}

pub(crate) fn get_name_from_template(
    template_name: &String,
    number: &NonZeroU16,
    presence: Option<Presence>,
    user_name: &str,
) -> String {
    let current_activity = get_presence_str(presence).unwrap_or_else(|| "No Game".to_string());

    template_name
        .replace("%number%", number.to_string().as_str())
        .replace("%name%", user_name)
        .replace("%room%", get_end_modifier(user_name))
        .replace("%current_activity%", current_activity.as_str())
}
pub(crate) fn get_user_presence(
    ctx: &Context,
    guild_id: &GuildId,
    user_id: &UserId,
) -> Option<Presence> {
    match guild_id.to_guild_cached(ctx) {
        None => None,
        Some(guild_ref) => {
            return match guild_ref.presences.get(user_id) {
                None => None,
                Some(presence) => Some(presence.to_owned()),
            }
        }
    }
}

fn get_presence_str(presence: Option<Presence>) -> Option<String> {
    match presence {
        None => None,
        Some(presence) => {
            // TODO: Improve this so that it gets more types and order them
            for activity in presence.activities {
                if activity.kind == ActivityType::Playing {
                    return Some(activity.name);
                }
            }

            return None;
        }
    }
}

fn normalize_char(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
        'Á' | 'À' | 'Ä' | 'Â' | 'Ã' | 'Å' | 'Ā' | 'Ă' | 'Ą' => 'A',
        'ß' => 'b',
        'ç' => 'c',
        'Ç' => 'C',
        'é' | 'è' | 'ë' | 'ê' | 'ę' | 'ė' | 'ē' => 'e',
        'É' | 'È' | 'Ë' | 'Ê' | 'Ę' | 'Ė' | 'Ē' => 'E',
        'í' | 'ì' | 'ï' | 'î' | 'į' | 'ī' | 'ᵢ' => 'i',
        'Í' | 'Ì' | 'Ï' | 'Î' | 'Į' | 'Ī' => 'I',
        'ñ' => 'n',
        'Ñ' | 'Ɲ' => 'N',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' | 'ø' | 'ō' | 'ő' => 'o',
        'Ó' | 'Ò' | 'Ö' | 'Ô' | 'Õ' | 'Ø' | 'Ō' | 'Ő' => 'O',
        'Ɽ' => 'R',
        'ú' | 'ù' | 'ü' | 'û' | 'ū' | 'ů' | 'ű' => 'u',
        'Ú' | 'Ù' | 'Ü' | 'Û' | 'Ū' | 'Ů' | 'Ű' => 'U',
        'ÿ' | 'ý' => 'y',
        'Ÿ' | 'Ý' => 'Y',
        _ => c,
    }
}

fn get_end_modifier(member_name: &str) -> &str {
    let first_char_of_member_name = member_name.chars().next().unwrap_or_default();

    let end_modifiers = get_end_modifiers(first_char_of_member_name);

    assert!(end_modifiers.len() > 0);

    let len = end_modifiers.len();
    let index = rand::prelude::thread_rng().gen_range(0..len);

    end_modifiers[index]
}

fn get_end_modifiers(first_char_of_member_name: char) -> Vec<&'static str> {
    let first_char_of_member_name = normalize_char(first_char_of_member_name);

    return match first_char_of_member_name.to_ascii_lowercase() {
        'a' => vec!["Atrium", "Arcade", "Arena", "Area"],
        'b' => vec!["Bureau", "Base", "Building"],
        'c' => vec!["Corner", "Court", "Cave", "City", "Cool-de-Sac", "Club", "Chill-Zone"],
        'd' => vec!["Domain", "Den", "Depot", "District"],
        'e' => vec!["Estate", "Embassy", "Entrance"],
        'f' => vec!["Fortress", "Farmhouse", "Factory"],
        'g' => vec!["Grounds", "Gallery", "Garden"],
        'h' => vec!["Haven", "Hall", "Harbor"],
        'i' => vec!["Institute", "Inn", "Island"],
        'j' => vec!["Junction", "Jungle"],
        'k' => vec!["Kingdom", "Keep", "Kitchen"],
        'l' => vec!["Loft", "Library", "Lodge"],
        'm' => vec!["Manor", "Museum", "Mill"],
        'n' => vec!["Nook", "Nest", "Nave"],
        'o' => vec!["Office", "Outpost", "Observatory"],
        'p' => vec!["Plaza", "Palace", "Parlor"],
        'q' => vec!["Quarters", "Quay", "Quadrangle"],
        'r' => vec!["Room", "Resort", "Retreat"],
        's' => vec!["Studio", "Sanctuary", "Store", "Sector", "Section"],
        't' => vec!["Territory", "Tower", "Temple"],
        'u' => vec!["University"],
        'v' => vec!["Villa", "Valley", "Vault"],
        'w' => vec!["Workshop", "Warehouse", "Wharf"],
        'x' => vec!["Xystus"],
        'y' => vec!["Yard", "Yacht", "Yardhouse"],
        'z' => vec!["Zone"],
        _ => vec!["VC"],
    };
}

#[cfg(test)]
mod tests {
    use crate::temporary_channel::{get_end_modifiers, get_name_from_template};
    use std::num::NonZeroU16;

    #[test]
    fn check_template_name_1() {
        let template_name = "%name% - %number%";
        let name = get_name_from_template(
            &template_name.to_string(),
            &NonZeroU16::new(83).unwrap(),
            None,
            "Inbound",
        );

        assert_eq!(name, "Inbound - 83")
    }

    #[test]
    fn check_template_name_2() {
        let template_name = "%name%'s %room%";
        let name = get_name_from_template(
            &template_name.to_string(),
            &NonZeroU16::new(42).unwrap(),
            None,
            "ⱤoᵀᴛᵥƝₓˣ",
        );

        let room = name.strip_prefix("ⱤoᵀᴛᵥƝₓˣ's "); // This was a user in a discord guild that did not have a normalised username
        assert_eq!(room.is_some(), true); // Assert that the prefix is "ⱤoᵀᴛᵥƝₓˣ's "

        assert!(get_end_modifiers('Ɽ').contains(&room.unwrap()));
    }
}
