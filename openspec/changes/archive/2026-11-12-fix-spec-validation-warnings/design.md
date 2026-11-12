# Design

## Context

The deployment spec's Purpose is a placeholder. The PDF ingestion spec has a heading requirement over 500 characters with seven existing scenarios. Both specs validate with warnings. See the proposal for the intended scope.

## Goals / Non-Goals

**Goals:**
- Make the two main specs clear and warning-free while preserving the existing contract.
- Make preservation of every heading behavior and scenario reviewable.

**Non-Goals:**
- Change the application, supported PDF formats, or deployment behavior.
- Introduce new behavioral requirements or modify unrelated specifications.

## Decisions

- Edit the deployment Purpose directly: delta spec Purpose text is only used for newly created capabilities. Describe the existing configurable API routing behavior rather than broadening the capability.
- Split the PDF heading requirement by behavior: detection and semantic hierarchy, ordinary prose and unsupported levels, and continuation of wrapped headings. Preserve every original statement and all seven scenario texts, placing each scenario under the most relevant requirement; retain the original requirement name for the core hierarchy behavior.
- Compare the revised statements and scenarios against the original main spec before accepting the rewrite, then run strict validation on all main specs. This retains the contract while avoiding a delta whose archive merge might fail to remove the original duplicated content.

## Risks / Trade-offs

- [A split can weaken a condition or lose a scenario] → Map every sentence and scenario to the revised requirements and review the diff for semantic equivalence.
- [Other warnings can arise independently] → Report validation output accurately; resolve only the two warnings in the approved scope.
