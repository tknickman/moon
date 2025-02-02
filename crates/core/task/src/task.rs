use crate::errors::TaskError;
use crate::task_options::TaskOptions;
use crate::types::TouchedFilePaths;
use moon_args::{split_args, ArgsSplitError};
use moon_common::{cacheable, cacheable_enum, Id};
use moon_config::{PlatformType, TaskCommandArgs, TaskConfig, TaskMergeStrategy, TaskType};
use moon_error::MoonError;
use moon_logger::{debug, trace, Logable};
use moon_target::Target;
use rustc_hash::{FxHashMap, FxHashSet};
use starbase_styles::color;
use starbase_utils::glob;
use std::env;
use std::path::PathBuf;

type EnvVars = FxHashMap<String, String>;

cacheable_enum!(
    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    pub enum TaskFlag {
        NoInputs,
    }
);

cacheable!(
    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub struct Task {
        pub args: Vec<String>,

        pub command: String,

        pub deps: Vec<Target>,

        pub env: EnvVars,

        pub flags: FxHashSet<TaskFlag>,

        pub global_inputs: Vec<String>,

        pub id: Id,

        pub inputs: Vec<String>,

        // Relative from workspace root
        pub input_globs: FxHashSet<String>,

        // Relative from workspace root
        pub input_paths: FxHashSet<PathBuf>,

        pub input_vars: FxHashSet<String>,

        #[serde(skip)]
        pub log_target: String,

        pub options: TaskOptions,

        pub outputs: Vec<String>,

        // Relative from workspace root
        pub output_globs: FxHashSet<String>,

        // Relative from workspace root
        pub output_paths: FxHashSet<PathBuf>,

        pub platform: PlatformType,

        pub target: Target,

        #[serde(rename = "type")]
        pub type_of: TaskType,
    }
);

impl Logable for Task {
    fn get_log_target(&self) -> &str {
        &self.log_target
    }
}

impl Task {
    pub fn from_config(target: Target, config: &TaskConfig) -> Result<Self, TaskError> {
        let cloned_config = config.clone();
        let cloned_options = cloned_config.options;

        let (command, args) = get_command_and_args(config)?;
        let command = command.unwrap_or_else(|| "noop".to_owned());
        let is_local =
            cloned_config.local || command == "dev" || command == "serve" || command == "start";
        let log_target = format!("moon:project:{}", target.id);

        debug!(
            target: &log_target,
            "Creating task {} with command {}",
            color::label(&target.id),
            color::shell(&command)
        );

        let mut task = Task {
            args,
            command,
            deps: cloned_config.deps,
            env: cloned_config.env,
            flags: FxHashSet::default(),
            global_inputs: cloned_config.global_inputs,
            id: Id::new(&target.task_id)?,
            inputs: cloned_config.inputs.unwrap_or_default(),
            input_vars: FxHashSet::default(),
            input_globs: FxHashSet::default(),
            input_paths: FxHashSet::default(),
            log_target,
            options: TaskOptions::from_config(cloned_options, is_local),
            outputs: cloned_config.outputs.unwrap_or_default(),
            output_globs: FxHashSet::default(),
            output_paths: FxHashSet::default(),
            platform: cloned_config.platform,
            target,
            type_of: if is_local {
                TaskType::Run
            } else {
                TaskType::Test
            },
        };

        if config
            .inputs
            .as_ref()
            .map(|i| i.is_empty())
            .unwrap_or(false)
        {
            task.flags.insert(TaskFlag::NoInputs);
        }

        Ok(task)
    }

    pub fn to_config(&self) -> TaskConfig {
        let mut command = vec![self.command.clone()];
        command.extend(self.args.clone());

        let mut config = TaskConfig {
            command: TaskCommandArgs::Sequence(command),
            options: self.options.to_config(),
            ..TaskConfig::default()
        };

        if !self.deps.is_empty() {
            config.deps = self.deps.clone();
        }

        if !self.env.is_empty() {
            config.env = self.env.clone();
        }

        if !self.inputs.is_empty() || (self.inputs.len() == 1 && self.inputs[0] == "**/*") {
            config.inputs = Some(self.inputs.clone());
        }

        if !self.outputs.is_empty() {
            config.outputs = Some(self.outputs.clone());
        }

        if !self.platform.is_unknown() {
            config.platform = self.platform;
        }

        config
    }

    /// Create a globset of all input globs to match with.
    pub fn create_globset(&self) -> Result<glob::GlobSet, TaskError> {
        Ok(
            glob::GlobSet::new_split(&self.input_globs, &self.output_globs)
                .map_err(MoonError::StarGlob)?,
        )
    }

    /// Determine the type of task after inheritance and expansion.
    pub fn determine_type(&mut self) {
        if !self.outputs.is_empty() {
            self.type_of = TaskType::Build;
        }
    }

    /// Return a list of affected files filtered down from the provided touched files list.
    pub fn get_affected_files(
        &self,
        touched_files: &TouchedFilePaths,
        project_source: &str,
    ) -> Result<Vec<PathBuf>, TaskError> {
        let mut files = vec![];
        let globset = self.create_globset()?;

        for file in touched_files {
            // Don't run on files outside of the project
            if !file.starts_with(project_source) {
                continue;
            }

            if self.input_paths.contains(file) || globset.matches(file) {
                // Mimic relative from ("./")
                files.push(PathBuf::from(".").join(file.strip_prefix(project_source).unwrap()));
            }
        }

        Ok(files)
    }

    /// Return true if this task is affected based on touched files.
    /// Will attempt to find any file that matches our list of inputs.
    pub fn is_affected(&self, touched_files: &TouchedFilePaths) -> Result<bool, TaskError> {
        // If an empty inputs ([]), we should always run
        if self.flags.contains(&TaskFlag::NoInputs) {
            return Ok(true);
        }

        for var_name in &self.input_vars {
            if let Ok(var) = env::var(var_name) {
                if !var.is_empty() {
                    trace!(
                        target: self.get_log_target(),
                        "Affected by {} (via environment variable)",
                        color::symbol(var_name),
                    );

                    return Ok(true);
                }
            }
        }

        let globset = self.create_globset()?;

        for file in touched_files {
            if self.input_paths.contains(file) {
                trace!(
                    target: self.get_log_target(),
                    "Affected by {} (via input files)",
                    color::path(file),
                );

                return Ok(true);
            }

            if globset.matches(file) {
                trace!(
                    target: self.get_log_target(),
                    "Affected by {} (via input globs)",
                    color::path(file),
                );

                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Return true if the task is a "build" type.
    pub fn is_build_type(&self) -> bool {
        matches!(self.type_of, TaskType::Build)
    }

    /// Return true if the task is a "no operation" and does nothing.
    pub fn is_no_op(&self) -> bool {
        self.command == "nop" || self.command == "noop" || self.command == "no-op"
    }

    /// Return true if the task is a "run" type.
    pub fn is_run_type(&self) -> bool {
        matches!(self.type_of, TaskType::Run)
    }

    /// Return true if the task is a "test" type.
    pub fn is_test_type(&self) -> bool {
        matches!(self.type_of, TaskType::Test)
    }

    pub fn is_persistent(&self) -> bool {
        self.options.persistent
    }

    pub fn merge(&mut self, config: &TaskConfig) -> Result<(), TaskError> {
        let (command, args) = get_command_and_args(config)?;

        // Merge options first incase the merge strategy has changed
        self.options.merge(&config.options);

        if !config.platform.is_unknown() {
            self.platform = config.platform;
        }

        // Then merge the actual task fields
        if let Some(cmd) = command {
            self.command = cmd;
        }

        if !args.is_empty() {
            self.args = self.merge_vec(&self.args, &args, &self.options.merge_args);
        }

        if !config.deps.is_empty() {
            self.deps =
                self.merge_vec::<Target>(&self.deps, &config.deps, &self.options.merge_deps);
        }

        if !config.env.is_empty() {
            self.env = self.merge_env_vars(&self.env, &config.env, &self.options.merge_env);
        }

        if let Some(inputs) = &config.inputs {
            if inputs.is_empty() {
                self.flags.insert(TaskFlag::NoInputs);
                self.inputs = vec![];
            } else {
                self.flags.remove(&TaskFlag::NoInputs);
                self.inputs = self.merge_vec(&self.inputs, inputs, &self.options.merge_inputs);
            }
        }

        if let Some(outputs) = &config.outputs {
            self.outputs = self.merge_vec(&self.outputs, outputs, &self.options.merge_outputs);
        }

        Ok(())
    }

    pub fn should_run_in_ci(&self) -> bool {
        if !self.options.run_in_ci {
            return false;
        }

        self.is_build_type() || self.is_test_type()
    }

    fn merge_env_vars(
        &self,
        base: &EnvVars,
        next: &EnvVars,
        strategy: &TaskMergeStrategy,
    ) -> EnvVars {
        match strategy {
            TaskMergeStrategy::Append => {
                let mut map = base.clone();
                map.extend(next.clone());
                map
            }
            TaskMergeStrategy::Prepend => {
                let mut map = next.clone();
                map.extend(base.clone());
                map
            }
            TaskMergeStrategy::Replace => next.clone(),
        }
    }

    fn merge_vec<T: Clone>(&self, base: &[T], next: &[T], strategy: &TaskMergeStrategy) -> Vec<T> {
        let mut list: Vec<T> = vec![];

        // This is easier than .extend() as we need to clone the inner value
        let mut merge = |inner_list: &[T]| {
            for item in inner_list {
                list.push(item.clone());
            }
        };

        match strategy {
            TaskMergeStrategy::Append => {
                merge(base);
                merge(next);
            }
            TaskMergeStrategy::Prepend => {
                merge(next);
                merge(base);
            }
            TaskMergeStrategy::Replace => {
                merge(next);
            }
        }

        list
    }
}

fn get_command_and_args(
    task: &TaskConfig,
) -> Result<(Option<String>, Vec<String>), ArgsSplitError> {
    let mut command = None;
    let mut args = vec![];

    let mut cmd_list = match &task.command {
        TaskCommandArgs::None => vec![],
        TaskCommandArgs::String(cmd_string) => split_args(cmd_string)?,
        TaskCommandArgs::Sequence(cmd_args) => cmd_args.clone(),
    };

    if !cmd_list.is_empty() {
        command = Some(cmd_list.remove(0));
        args.extend(cmd_list);
    }

    match &task.args {
        TaskCommandArgs::None => {}
        TaskCommandArgs::String(args_string) => args.extend(split_args(args_string)?),
        TaskCommandArgs::Sequence(args_list) => args.extend(args_list.clone()),
    }

    Ok((command, args))
}
