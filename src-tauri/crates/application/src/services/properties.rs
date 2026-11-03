use std::sync::Arc;

use domain::property::Property;
use domain::repositories::{MachineRepository, PropertyRepository};
use domain::DomainError;

use crate::dto::PropertyInput;
use crate::error::logged;
use crate::AppResult;

pub struct PropertyService {
    machines: Arc<dyn MachineRepository>,
    properties: Arc<dyn PropertyRepository>,
}

impl PropertyService {
    pub fn new(machines: Arc<dyn MachineRepository>, properties: Arc<dyn PropertyRepository>) -> Self {
        Self { machines, properties }
    }

    pub fn list(&self, machine_id: i64) -> AppResult<Vec<Property>> {
        logged("properties.list", self.properties.list_by_machine(machine_id).map_err(Into::into))
    }

    pub fn create(&self, input: PropertyInput) -> AppResult<Property> {
        logged("properties.create", self.save(None, input))
    }

    pub fn update(&self, id: i64, input: PropertyInput) -> AppResult<Property> {
        logged("properties.update", self.save(Some(id), input))
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        logged("properties.delete", self.properties.delete(id).map_err(Into::into))
    }

    fn save(&self, id: Option<i64>, input: PropertyInput) -> AppResult<Property> {
        self.machines.get(input.machine_id)?.ok_or(DomainError::NotFound { entity: "machine" })?;
        let existing = match id {
            Some(id) => Some(self.properties.get(id)?.ok_or(DomainError::NotFound { entity: "property" })?),
            None => None,
        };
        let sort_order = match (input.sort_order, &existing) {
            (Some(s), _) => s,
            (None, Some(e)) => e.sort_order,
            (None, None) => self.next_sort_order(input.machine_id)?,
        };
        let mut property = Property {
            id,
            machine_id: input.machine_id,
            code: input.code.trim().to_uppercase(),
            name: input.name.trim().to_string(),
            unit: input.unit.trim().to_string(),
            decimals: input.decimals,
            sort_order,
            active: input.active,
            limits: input.limits,
        };
        property.validate()?;
        if let Some(other) = self.properties.find_by_code(property.machine_id, &property.code)? {
            if other.id != id {
                return Err(DomainError::Conflict { code: "propertyCodeExists" }.into());
            }
        }
        match id {
            Some(_) => self.properties.update(&property)?,
            None => property.id = Some(self.properties.insert(&property)?),
        }
        Ok(property)
    }

    fn next_sort_order(&self, machine_id: i64) -> AppResult<i32> {
        let list = self.properties.list_by_machine(machine_id)?;
        Ok(list.iter().map(|p| p.sort_order).max().unwrap_or(0) + 1)
    }
}
