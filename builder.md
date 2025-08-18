# Builder Guide: Writing Effective Project Documentation

## Overview

This guide provides a systematic approach to creating **requisites.md**, **design.md**, and **plan-tasks.md** using **bottom-up development**, **DRY principles**, and **minimalist approaches**. Based on modern AI-assisted development practices and Rust idioms.

---

## 1. Core Principles

### 1.1 Bottom-Up Development Philosophy

**Start with atomic components, build upward:**
- Begin with smallest, testable units
- Define foundational data structures first
- Compose complex behaviors from simple building blocks
- Validate each layer before adding the next

### 1.2 DRY (Don't Repeat Yourself) Application

**Eliminate redundancy across documents:**
- Reference shared concepts between documents
- Use consistent terminology throughout
- Create reusable templates and patterns
- Link related sections instead of duplicating

### 1.3 Minimalist Approach

**Essential information only:**
- Focus on actionable content
- Remove unnecessary elaboration
- Use clear, concise language
- Prioritize scannable structure

---

## 2. Document Structure Templates

### 2.1 Requisites.md Template

```markdown
# Requirements Document - [Project Name]

## 1. Project Overview
- **Name:** [Clear, descriptive name]
- **Purpose:** [One sentence goal]
- **Users:** [Primary stakeholders]

## 2. Functional Requirements
### 2.1 Core Features
- **FR-001:** [Atomic functionality]
- **FR-002:** [Next atomic functionality]

### 2.2 [Feature Group]
- **FR-XXX:** [Specific, testable requirement]

## 3. Non-Functional Requirements
### 3.1 Performance
- **NFR-001:** [Measurable performance target]

### 3.2 Usability
- **NFR-XXX:** [User experience requirement]

## 4. Technical Constraints
- **TC-001:** [Technology limitation or choice]

## 5. Success Criteria
- **SC-001:** [Measurable success metric]
```

### 2.2 Design.md Template

```markdown
# Design Document - [Project Name]

## 1. System Architecture
[High-level component diagram]

## 2. Core Data Structures
```rust
// Start with atomic types
pub struct [AtomicType] {
    // Minimal essential fields
}

// Build composite types
pub struct [CompositeType] {
    // Composed from atomic types
}
```

## 3. Component Design
### 3.1 [Component Name]
**Responsibility:** [Single responsibility]
**Key Classes:** [List main classes]
**Design Patterns:** [Applied patterns]

## 4. Algorithm Design
[Core algorithms with complexity analysis]

## 5. Performance Considerations
[Optimization strategies]
```

### 2.3 Plan-Tasks.md Template

```markdown
# Project Plan & Tasks - [Project Name]

## 1. Phase Overview
### Phase 1: Foundation (Week X-Y)
**Goal:** [Atomic deliverable]

## 2. Task Breakdown
### Phase 1: Foundation
- [ ] **T001** - [Atomic task]
  - **Priority:** [High/Medium/Low]
  - **Estimate:** [Hours]
  - **Dependencies:** [Task IDs]
  - **Deliverable:** [Testable output]

## 3. Critical Path
[Task dependency chain]

## 4. Quality Gates
[Phase completion criteria]
```

---

## 3. Bottom-Up Documentation Strategy

### 3.1 Start with Data (Requisites.md)

**Step 1: Identify Atomic Requirements**
```markdown
# Start with smallest, testable units
- Parse single Rust function
- Extract one item type (struct)
- Store in basic data structure
- Output to console

# Build upward
- Parse multiple item types
- Build relationships
- Query capabilities
- Visualization
```

**Step 2: Group Related Atomics**
```markdown
# Group by natural boundaries
## Core Analysis (atomic parsing functions)
## Query System (atomic query operations)  
## Visualization (atomic output formats)
```

**Step 3: Define Measurable Success**
```markdown
# Each requirement must be:
- Testable with clear pass/fail
- Independent of other requirements
- Implementable in isolation
- Valuable to end users
```

### 3.2 Design from Components (Design.md)

**Step 1: Define Atomic Data Structures**
```rust
// Start with smallest meaningful units
#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub item_type: ItemType,
}

// Compose upward
pub struct FileNode {
    pub items: Vec<Item>,  // Composition, not inheritance
}

pub struct KnowledgeGraph {
    pub files: HashMap<PathBuf, FileNode>,  // Bottom-up assembly
}
```

**Step 2: Single Responsibility Components**
```markdown
## Parser Component
**Responsibility:** Extract items from text (ONE thing)
**Input:** String content
**Output:** Vec<Item>

## Graph Component  
**Responsibility:** Manage relationships (ONE thing)
**Input:** Vec<FileNode>
**Output:** KnowledgeGraph
```

**Step 3: Compose Behaviors**
```rust
// Build complex operations from simple ones
impl KnowledgeGraph {
    // Atomic operation
    fn add_relationship(&mut self, rel: Relationship) { }
    
    // Composed operation
    fn analyze_dependencies(&self) -> Vec<Dependency> {
        // Uses multiple atomic operations
    }
}
```

### 3.3 Plan from Deliverables (Plan-Tasks.md)

**Step 1: Atomic Tasks**
```markdown
# Each task produces ONE testable deliverable
- [ ] T001 - Parse single function (2h) → Working function parser
- [ ] T002 - Parse single struct (2h) → Working struct parser  
- [ ] T003 - Combine parsers (1h) → Multi-type parser

# NOT: "Implement parsing system" (too big, not atomic)
```

**Step 2: Dependency Chains**
```markdown
# Build dependency graph bottom-up
T001 (atomic) → T003 (composition) → T005 (integration)
T002 (atomic) → T003 (composition) → T005 (integration)

# Parallel opportunities
T001 + T002 can run in parallel
T004 (data structures) can run parallel to T001-T003
```

**Step 3: Incremental Value**
```markdown
# Each phase delivers working software
Phase 1: Basic parsing works (can demo)
Phase 2: Relationships work (can query)
Phase 3: Visualization works (can see results)
```

---

## 4. DRY Implementation Across Documents

### 4.1 Shared Terminology Dictionary

**Create once, reference everywhere:**
```markdown
# In requisites.md
**Knowledge Graph:** Directed graph representing code relationships

# In design.md  
// References the same definition
pub struct KnowledgeGraph { ... }  // See requisites.md definition

# In plan-tasks.md
T010 - Build KnowledgeGraph (see requisites.md FR-010)
```

### 4.2 Cross-Document References

**Link instead of duplicate:**
```markdown
# requisites.md
## 2.1 Core Analysis Features
- **FR-001:** Parse Rust items (see design.md Section 3.1)

# design.md  
## 3.1 Parser Component
**Requirements:** Implements FR-001 from requisites.md

# plan-tasks.md
- [ ] T008 - Implement item extraction (addresses FR-001)
```

### 4.3 Template Reuse

**Standardize patterns:**
```markdown
# Requirement pattern (reuse for all FRs)
- **[ID]:** [Action] [Object] [Constraint]
  - **Input:** [What goes in]
  - **Output:** [What comes out]  
  - **Success:** [How to verify]

# Task pattern (reuse for all tasks)
- [ ] **[ID]** - [Deliverable] 
  - **Priority:** [Level]
  - **Estimate:** [Time]
  - **Dependencies:** [Prerequisites]
  - **Deliverable:** [Testable output]
```

---

## 5. Minimalist Content Guidelines

### 5.1 Essential Information Only

**Include:**
- Actionable requirements
- Testable specifications  
- Clear dependencies
- Measurable outcomes

**Exclude:**
- Background theory
- Obvious statements
- Redundant explanations
- Implementation details in requirements

### 5.2 Scannable Structure

**Use consistent formatting:**
```markdown
# Level 1: Major sections
## Level 2: Functional groups  
### Level 3: Specific items

- **Bold:** Key terms and IDs
- `Code:` Technical references
- [Links]: Cross-references
```

### 5.3 Concise Language

**Write clearly:**
```markdown
# Good (specific, actionable)
- **FR-001:** Parse function declarations returning Function struct

# Bad (vague, wordy)  
- **FR-001:** The system should be able to analyze and understand 
  Rust source code files in order to extract information about 
  function definitions and create appropriate data structures
```

---

## 6. Quality Checklist

### 6.1 Requisites.md Quality Gates

- [ ] Each requirement is atomic and testable
- [ ] Requirements grouped by natural boundaries
- [ ] Success criteria are measurable
- [ ] No implementation details leaked
- [ ] All requirements traceable to user value

### 6.2 Design.md Quality Gates

- [ ] Data structures follow composition over inheritance
- [ ] Components have single responsibilities  
- [ ] Algorithms specified with complexity
- [ ] Performance considerations addressed
- [ ] Extension points identified

### 6.3 Plan-Tasks.md Quality Gates

- [ ] Tasks produce atomic deliverables
- [ ] Dependencies clearly mapped
- [ ] Estimates based on atomic work
- [ ] Critical path identified
- [ ] Quality gates defined per phase

---

## 7. AI-Assisted Development Integration

### 7.1 LLM-Friendly Structure

**Use consistent patterns:**
```markdown
# Pattern for requirements
**[ID]:** [System] shall [action] [object] [constraint]

# Pattern for tasks  
**[ID]** - [Deliverable] ([estimate]) → [testable output]

# Pattern for components
**[Name]:** [responsibility] | Input: [type] | Output: [type]
```

### 7.2 Context Establishment

**Provide clear context blocks:**
```markdown
## Context Block
**Project:** Rust Knowledge Graph System
**Phase:** Foundation Setup  
**Dependencies:** Cargo project initialized
**Goal:** Working parser for basic Rust constructs
```

### 7.3 Validation Prompts

**Include verification guidance:**
```markdown
## Validation Checklist
- [ ] Can parse sample Rust file
- [ ] Extracts functions and structs
- [ ] Handles visibility modifiers
- [ ] Returns structured data
- [ ] Error handling works
```

---

## 8. Common Anti-Patterns to Avoid

### 8.1 Requirements Anti-Patterns

**❌ Avoid:**
- Mixing requirements with implementation
- Vague, untestable statements
- Duplicate information across sections
- Requirements that depend on multiple systems

**✅ Instead:**
- Pure functional specifications
- Specific, measurable criteria
- Single source of truth with references
- Atomic, independent requirements

### 8.2 Design Anti-Patterns

**❌ Avoid:**
- God objects with multiple responsibilities
- Deep inheritance hierarchies
- Tight coupling between components
- Premature optimization

**✅ Instead:**
- Single responsibility principle
- Composition over inheritance
- Loose coupling with clear interfaces
- Performance considerations documented

### 8.3 Planning Anti-Patterns

**❌ Avoid:**
- Tasks that can't be completed in isolation
- Estimates without breaking down work
- Dependencies that create circular waits
- Phases without deliverable value

**✅ Instead:**
- Atomic, testable deliverables
- Bottom-up estimation from known work
- Clear dependency chains
- Incremental value delivery

---

## 9. Example Application

### 9.1 Bottom-Up Requirement Building

```markdown
# Start atomic
FR-001: Parse function signature → Function struct
FR-002: Parse struct definition → Struct struct  
FR-003: Parse enum definition → Enum struct

# Compose upward
FR-010: Parse Rust file → Vec<Item> (uses FR-001, FR-002, FR-003)
FR-020: Build file relationships → KnowledgeGraph (uses FR-010)
FR-030: Query relationships → QueryResult (uses FR-020)
```

### 9.2 DRY Design References

```markdown
# design.md references requisites.md
## Parser Component (implements FR-001, FR-002, FR-003)
pub struct RustParser {
    // Implementation details
}

# plan-tasks.md references both
T008 - Implement RustParser (FR-001-003, see design.md Section 3.1)
```

### 9.3 Minimalist Task Definition

```markdown
# Atomic and specific
- [ ] **T008** - Function signature parsing (4h)
  - **Dependencies:** T004 (data structures)
  - **Deliverable:** Parse "fn name() -> Type" → Function struct
  - **Test:** Parse sample functions, verify struct fields
```

---

## 10. Success Metrics

### 10.1 Documentation Quality

- **Completeness:** All requirements traceable to tasks
- **Clarity:** Technical reviewer can implement from docs
- **Maintainability:** Updates require minimal changes
- **Usability:** AI can generate code from specifications

### 10.2 Development Efficiency

- **Reduced Rework:** Clear requirements prevent misunderstanding
- **Faster Implementation:** Atomic tasks enable parallel work
- **Better Testing:** Specific criteria enable thorough validation
- **Easier Maintenance:** Modular design supports evolution

### 10.3 Team Collaboration

- **Shared Understanding:** Consistent terminology across team
- **Clear Ownership:** Atomic tasks have clear responsibility
- **Progress Visibility:** Deliverables show concrete progress
- **Quality Assurance:** Built-in validation at each step
