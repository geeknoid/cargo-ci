use crate::config::{Job, JobId};
use serde::Deserialize;
use serde::de::{self, Deserializer};
use std::collections::HashMap;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Default)]
pub struct Jobs(HashMap<JobId, Job>);

impl Jobs {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_job(&self, id: &JobId) -> Option<&Job> {
        self.0.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&JobId, &Job)> {
        self.0.iter()
    }

    pub fn topological_sort(&self, jobs_to_sort: &HashSet<&JobId>) -> Vec<&JobId> {
        let mut in_degree: HashMap<&JobId, usize> = jobs_to_sort.iter().map(|&id| (id, 0)).collect();

        for &job_id in jobs_to_sort {
            if let Some(job) = self.get_job(job_id) {
                for needed in job.needs() {
                    if jobs_to_sort.contains(&needed) {
                        *in_degree.entry(job_id).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut queue: VecDeque<&JobId> = in_degree.iter().filter(|(_, degree)| **degree == 0).map(|(id, _)| *id).collect();

        let mut sorted_jobs = Vec::new();
        while let Some(job_id) = queue.pop_front() {
            // Look up the job_id from self to get the correct lifetime
            if let Some((actual_job_id, _)) = self.0.get_key_value(job_id) {
                sorted_jobs.push(actual_job_id);
            }

            for (other_job_id, other_job) in self.iter() {
                if other_job.needs().contains(job_id)
                    && let Some(&job_id_ref) = jobs_to_sort.get(other_job_id)
                    && let Some(degree) = in_degree.get_mut(&job_id_ref)
                {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(job_id_ref);
                    }
                }
            }
        }

        sorted_jobs
    }

    pub fn get_transitive_needs(&self, job_id: &JobId) -> Vec<&JobId> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(job) = self.get_job(job_id) {
            for needed_job_id in job.needs() {
                queue.push_back(needed_job_id);
            }
        }

        while let Some(current_job_id) = queue.pop_front() {
            if visited.contains(current_job_id) {
                continue;
            }
            _ = visited.insert(current_job_id);
            result.push(current_job_id);

            if let Some(current_job) = self.get_job(current_job_id) {
                for needed_job_id in current_job.needs() {
                    if !visited.contains(needed_job_id) {
                        queue.push_back(needed_job_id);
                    }
                }
            }
        }

        result
    }
}

impl<'de> Deserialize<'de> for Jobs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let jobs_map: HashMap<JobId, Job> = HashMap::deserialize(deserializer)?;

        for (job_id, job) in &jobs_map {
            // check for unknown dependencies
            for needed_job_id in job.needs() {
                if !jobs_map.contains_key(needed_job_id) {
                    return Err(de::Error::custom(format!(
                        "job '{job_id}' needs job '{needed_job_id}', but there is no '{needed_job_id}' job",
                    )));
                }
            }

            // check for duplicate step ids
            let mut seen = HashSet::new();
            for step in job.steps() {
                if let Some(id) = step.id()
                    && !seen.insert(id)
                {
                    return Err(de::Error::custom(format!("duplicate step id '{id}' found in job '{job_id}'")));
                }
            }
        }

        let mut visited = HashMap::new();
        for job_id in jobs_map.keys() {
            if !visited.contains_key(job_id) {
                let mut path = Vec::new();
                if let Err(e) = detect_cycle(job_id, &jobs_map, &mut visited, &mut path) {
                    return Err(de::Error::custom(e));
                }
            }
        }

        Ok(Self(jobs_map))
    }
}

fn detect_cycle<'a>(
    job_id: &'a JobId,
    jobs_map: &'a HashMap<JobId, Job>,
    visited: &mut HashMap<&'a JobId, bool>,
    path: &mut Vec<&'a JobId>,
) -> Result<(), String> {
    path.push(job_id);
    _ = visited.insert(job_id, true);

    if let Some(job) = jobs_map.get(job_id) {
        for needed_job_id in job.needs() {
            if path.contains(&needed_job_id) {
                let cycle_path = path.iter().map(ToString::to_string).collect::<Vec<_>>().join(" -> ");
                return Err(format!("circular dependency detected: {cycle_path} -> {needed_job_id}"));
            }

            if !visited.get(needed_job_id).copied().unwrap_or(false) {
                detect_cycle(needed_job_id, jobs_map, visited, path)?;
            }
        }
    }

    _ = path.pop();
    Ok(())
}
