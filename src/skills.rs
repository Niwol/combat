use std::collections::HashMap;

use bevy::prelude::*;

pub struct SkillPlugin;

impl Plugin for SkillPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tick_skill_timers);
    }
}

#[derive(Component, Default)]
pub struct SkillTree {
    skills: HashMap<String, Skill>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Skill {
    pub name: String,
    pub mana_cost: i32,
    pub reload_timer: Timer,
    pub unlocked: bool,
}

impl SkillTree {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    pub fn unlock_skill(&mut self, skill_name: String) {
        if let Some(skill) = self.skills.get_mut(&skill_name) {
            skill.unlocked = true;
        }
    }

    pub fn skill(&self, skill_name: String) -> &Skill {
        &self.skills[&skill_name]
    }

    pub fn skill_mut(&mut self, skill_name: String) -> &mut Skill {
        self.skills.get_mut(&skill_name).unwrap()
    }

    pub fn unlocked(&self, skill_name: String) -> bool {
        if let Some(skill) = self.skills.get(&skill_name) {
            return skill.unlocked;
        }

        false
    }
}

fn tick_skill_timers(time: Res<Time>, mut skill_trees: Query<&mut SkillTree>) {
    for mut skill_tree in &mut skill_trees {
        for skill in skill_tree.skills.values_mut() {
            skill.reload_timer.tick(time.delta());
        }
    }
}
