use crate::{
    drone::{DroneCatalog, DroneTask, FlightMode},
    errors::AerisError,
};

use super::MissionConfig;

pub struct MissionValidator;

impl MissionValidator {
    pub fn validate(
        mission: &MissionConfig,
        drone_catalog: &DroneCatalog,
    ) -> Result<(), AerisError> {
        if mission.name.trim().is_empty() {
            return Err(AerisError::InvalidMission(
                "mission name cannot be empty".to_string(),
            ));
        }

        if mission.groups.is_empty() {
            return Err(AerisError::InvalidMission(
                "mission must contain at least one group".to_string(),
            ));
        }

        for group in &mission.groups {
            if group.drone_names.is_empty() {
                return Err(AerisError::InvalidMission(format!(
                    "group '{}' must contain at least one drone",
                    group.drone_type
                )));
            }

            if group.tasks.is_empty() {
                return Err(AerisError::InvalidMission(format!(
                    "group '{}' must contain at least one task",
                    group.drone_type
                )));
            }

            let drone_config = drone_catalog
                .drones
                .iter()
                .find(|drone| drone.name == group.drone_type)
                .ok_or_else(|| AerisError::DroneTypeNotFound(group.drone_type.clone()))?;

            if !drone_config.battery_capacity.is_finite() || drone_config.battery_capacity <= 0.0 {
                return Err(AerisError::InvalidMission(format!(
                    "drone '{}' must have a positive battery capacity",
                    group.drone_type
                )));
            }

            if !drone_config.consumption_per_second.is_finite()
                || drone_config.consumption_per_second < 0.0
            {
                return Err(AerisError::InvalidMission(format!(
                    "drone '{}' must have non-negative battery consumption",
                    group.drone_type
                )));
            }

            let mut mode = FlightMode::Armed;

            for (index, task) in group.tasks.iter().enumerate() {
                let next_mode = match (&mode, task) {
                    (FlightMode::Armed, DroneTask::Takeoff { target_altitude }) => {
                        if !target_altitude.is_finite()
                            || *target_altitude <= 0.0
                            || *target_altitude > drone_config.max_altitude
                        {
                            return Err(AerisError::InvalidMission(format!(
                                "task {index} has invalid target altitude {target_altitude} for drone '{}'",
                                group.drone_type
                            )));
                        }

                        FlightMode::Hold
                    }
                    (FlightMode::Hold, DroneTask::Hold) => FlightMode::Hold,
                    (FlightMode::Hold, DroneTask::FlyTo { target }) => {
                        let latitude = target.latitude();
                        let longitude = target.longitude();

                        if !latitude.is_finite()
                            || !longitude.is_finite()
                            || !(-90.0..=90.0).contains(&latitude)
                            || !(-180.0..=180.0).contains(&longitude)
                        {
                            return Err(AerisError::InvalidMission(format!(
                                "task {index} has invalid target coordinates ({latitude}, {longitude}) for drone '{}'",
                                group.drone_type
                            )));
                        }

                        if *target == group.home_position {
                            FlightMode::ReturnToHome
                        } else {
                            FlightMode::Hold
                        }
                    }
                    (FlightMode::Hold, DroneTask::ReturnHome) => FlightMode::ReturnToHome,
                    (FlightMode::Hold | FlightMode::ReturnToHome, DroneTask::Land) => {
                        FlightMode::Idle
                    }
                    _ => {
                        return Err(AerisError::InvalidMission(format!(
                            "task {index} ({task:?}) cannot start from mode {mode:?} for drone '{}'",
                            group.drone_type
                        )));
                    }
                };

                mode = next_mode;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coordinates::Coordinates, drone::DroneConfig};

    use super::super::config::MissionGroupConfig;

    fn drone_catalog() -> DroneCatalog {
        DroneCatalog {
            drones: vec![DroneConfig {
                name: "scout".to_string(),
                max_speed: 18.0,
                climb_speed: 3.0,
                descent_speed: 2.0,
                max_altitude: 120.0,
                battery_capacity: 100.0,
                consumption_per_second: 0.1,
            }],
        }
    }

    fn mission(tasks: Vec<DroneTask>) -> MissionConfig {
        MissionConfig {
            name: "test".to_string(),
            random_failure: false,
            groups: vec![MissionGroupConfig {
                drone_type: "scout".to_string(),
                drone_names: vec!["DR-SCO-01".to_string()],
                home_position: Coordinates::new(50.4501, 30.5234),
                tasks,
            }],
        }
    }

    #[test]
    fn accepts_valid_task_transitions() {
        let mission = mission(vec![
            DroneTask::Takeoff {
                target_altitude: 50.0,
            },
            DroneTask::FlyTo {
                target: Coordinates::new(50.4510, 30.5240),
            },
            DroneTask::FlyTo {
                target: Coordinates::new(50.4520, 30.5250),
            },
            DroneTask::FlyTo {
                target: Coordinates::new(50.4501, 30.5234),
            },
            DroneTask::Land,
        ]);

        assert!(MissionValidator::validate(&mission, &drone_catalog()).is_ok());
    }

    #[test]
    fn rejects_invalid_task_transition() {
        let mission = mission(vec![DroneTask::Land]);

        assert!(matches!(
            MissionValidator::validate(&mission, &drone_catalog()),
            Err(AerisError::InvalidMission(_))
        ));
    }

    #[test]
    fn rejects_altitude_above_drone_limit() {
        let mission = mission(vec![DroneTask::Takeoff {
            target_altitude: 121.0,
        }]);

        assert!(matches!(
            MissionValidator::validate(&mission, &drone_catalog()),
            Err(AerisError::InvalidMission(_))
        ));
    }

    #[test]
    fn rejects_invalid_fly_to_coordinates() {
        let mission = mission(vec![
            DroneTask::Takeoff {
                target_altitude: 50.0,
            },
            DroneTask::FlyTo {
                target: Coordinates::new(91.0, 30.5234),
            },
        ]);

        assert!(matches!(
            MissionValidator::validate(&mission, &drone_catalog()),
            Err(AerisError::InvalidMission(_))
        ));
    }

    #[test]
    fn rejects_non_positive_battery_capacity() {
        let mut catalog = drone_catalog();
        catalog.drones[0].battery_capacity = 0.0;

        let mission = mission(vec![DroneTask::Takeoff {
            target_altitude: 50.0,
        }]);

        assert!(matches!(
            MissionValidator::validate(&mission, &catalog),
            Err(AerisError::InvalidMission(_))
        ));
    }

    #[test]
    fn rejects_negative_battery_consumption() {
        let mut catalog = drone_catalog();
        catalog.drones[0].consumption_per_second = -0.1;

        let mission = mission(vec![DroneTask::Takeoff {
            target_altitude: 50.0,
        }]);

        assert!(matches!(
            MissionValidator::validate(&mission, &catalog),
            Err(AerisError::InvalidMission(_))
        ));
    }
}
