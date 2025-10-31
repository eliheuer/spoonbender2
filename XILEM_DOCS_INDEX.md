# Xilem Placehero Architecture - Complete Documentation Index

This directory contains comprehensive documentation on how the Xilem Placehero example application is structured and organized, with specific guidance for refactoring Spoonbender to follow the same idiomatic patterns.

## Documentation Files

### 1. REFACTORING_SUMMARY.md (START HERE)
**Purpose**: Quick-start guide for refactoring Spoonbender

**Contains**:
- Current problems in Spoonbender
- Target architecture after refactoring
- 6-phase refactoring plan with code examples
- Key patterns to adopt
- Migration checklist
- Estimated effort (12-21 hours)
- Testing strategy

**When to use**: Start here first! This gives you the roadmap and action items.

**Key sections**:
- Current Problems section explains what needs fixing
- Refactoring Steps walks through each phase
- Key Patterns to Adopt shows the idiomatic way
- Migration Checklist helps track progress

---

### 2. XILEM_ARCHITECTURE.md (DETAILED REFERENCE)
**Purpose**: Complete architectural analysis of Placehero application

**Contains**:
- File organization overview
- Entry point pattern (main.rs)
- State definition pattern (lib.rs)
- View organization with root dispatcher
- Action/navigation pattern
- Component/view module structure
- Async/resource pattern
- State update patterns
- Module structure principles
- Xilem-specific patterns (use<>, fork, lens, map_action, OneOf)
- Complete refactoring guide for Spoonbender
- Code examples and patterns

**When to use**: Deep dive into understanding how everything works

**Key sections**:
- Sections 1-10 explain Placehero's structure
- Sections 11-12 explain Xilem patterns and refactoring
- Section 13 has sidebar pattern example for Spoonbender

---

### 3. XILEM_CODE_PATTERNS.md (COPY-PASTE REFERENCE)
**Purpose**: Exact code examples from Placehero to use as templates

**Contains**:
- 12 complete code examples:
  1. Minimal main.rs (11 lines)
  2. State definition in lib.rs
  3. Root view dispatcher (select_app)
  4. Main view composition (app_logic)
  5. View methods (sidebar, main_view)
  6. Action enum (actions.rs)
  7. Async task pattern (one-shot with task_raw)
  8. Async worker pattern (streaming with worker_raw)
  9. Resource provider pattern (avatars.rs)
  10. Conditional rendering (OneOf pattern)
  11. Component function pattern
  12. Module export pattern
- Refactoring template for Spoonbender at the end

**When to use**: When implementing specific patterns - copy the examples

**Example usage**:
"How do I structure async loading?" -> See section 7 (task_raw)
"How do I make conditional views?" -> See section 10 (OneOf)
"How do I do resource caching?" -> See section 9 (Avatars provider)

---

### 4. PLACEHERO_STRUCTURE.txt (VISUAL DIAGRAMS)
**Purpose**: ASCII diagrams showing Placehero's architecture

**Contains**:
- Entry point flow diagram
- State structure with field descriptions
- View composition hierarchy
- Action flow and handling
- Module organization with file list
- Resource provider pattern flow
- Async task pattern flow
- Conditional rendering examples
- Key Xilem combinators used
- Error handling patterns
- Refactoring checklist for Spoonbender
- Pattern checklist
- Suggested file sizes based on Placehero

**When to use**: When you need to visualize how things connect

**Visual references**:
- Shows exact data flow from action to state update
- Shows module hierarchy and dependencies
- Shows how fork(), lens(), map_action() work together

---

## Quick Navigation Guide

### By Task

**"I need to understand the overall architecture"**
1. Start: REFACTORING_SUMMARY.md - Current Problems section
2. Visual: PLACEHERO_STRUCTURE.txt - Entry Point Flow and State Structure
3. Deep dive: XILEM_ARCHITECTURE.md - Sections 1-5

**"I need to refactor my code"**
1. Start: REFACTORING_SUMMARY.md - entire document
2. Code examples: XILEM_CODE_PATTERNS.md - Refactoring Template section
3. Detailed patterns: XILEM_ARCHITECTURE.md - Sections 6-11
4. Checklist: PLACEHERO_STRUCTURE.txt - Refactoring Checklist

**"I need to understand a specific pattern"**

- **Main.rs minimal entry point**: XILEM_CODE_PATTERNS.md section 1
- **State definitions**: XILEM_CODE_PATTERNS.md section 2
- **Root view dispatcher**: XILEM_CODE_PATTERNS.md section 3
- **Main view composition**: XILEM_CODE_PATTERNS.md section 4
- **View methods**: XILEM_CODE_PATTERNS.md section 5
- **Actions**: XILEM_CODE_PATTERNS.md section 6
- **Async tasks**: XILEM_CODE_PATTERNS.md sections 7-8
- **Resource providers**: XILEM_CODE_PATTERNS.md section 9
- **Conditional rendering**: XILEM_CODE_PATTERNS.md section 10
- **Components**: XILEM_CODE_PATTERNS.md section 11

**"How does fork() work?"**: XILEM_ARCHITECTURE.md section 11.2 or PLACEHERO_STRUCTURE.txt - Key Xilem Combinators

**"How does lens() work?"**: XILEM_ARCHITECTURE.md section 4 or XILEM_CODE_PATTERNS.md section 3

**"What are OneOf types?"**: XILEM_ARCHITECTURE.md section 9 or XILEM_CODE_PATTERNS.md section 10

### By Document Length

- **Quick read (10 min)**: REFACTORING_SUMMARY.md - Target Architecture and Key Patterns
- **Medium read (20 min)**: PLACEHERO_STRUCTURE.txt - Module Organization and Patterns
- **Deep read (45 min)**: XILEM_ARCHITECTURE.md - complete document
- **Reference only**: XILEM_CODE_PATTERNS.md - use as needed for specific patterns

---

## File Organization Reference

After refactoring, your Spoonbender should have:

```
src/
├── main.rs              # From XILEM_CODE_PATTERNS.md section 1
├── lib.rs               # From XILEM_CODE_PATTERNS.md sections 2-5
├── actions.rs           # From XILEM_CODE_PATTERNS.md section 6
├── views/
│   ├── sidebar.rs
│   ├── canvas.rs
│   └── menu.rs
├── components/
│   ├── glyph_widget.rs  # Pattern from section 11
│   ├── outline.rs
│   └── metrics.rs
└── async_tasks/
    ├── loader.rs        # Pattern from section 7
    ├── saver.rs
    └── metrics.rs
```

See REFACTORING_SUMMARY.md for phase-by-phase instructions on creating this structure.

---

## Key Patterns Summary

All documentation files contain these patterns. Quick reference:

| Pattern | Where | When to use |
|---------|-------|------------|
| **Minimal main.rs** | XILEM_CODE_PATTERNS.md #1 | Always |
| **State in lib.rs** | XILEM_CODE_PATTERNS.md #2 | Always |
| **Root dispatcher** | XILEM_CODE_PATTERNS.md #3 | Always |
| **Main composition** | XILEM_CODE_PATTERNS.md #4 | Always |
| **View methods** | XILEM_CODE_PATTERNS.md #5 | Always |
| **Action enum** | XILEM_CODE_PATTERNS.md #6 | Always |
| **task_raw** | XILEM_CODE_PATTERNS.md #7 | One-shot async |
| **worker_raw** | XILEM_CODE_PATTERNS.md #8 | Streaming async |
| **Resource provider** | XILEM_CODE_PATTERNS.md #9 | Shared state (caching) |
| **OneOf** | XILEM_CODE_PATTERNS.md #10 | Conditional rendering |
| **Components** | XILEM_CODE_PATTERNS.md #11 | Reusable UI pieces |

---

## Placehero Example File Sizes

For reference, actual sizes from Placehero:

- main.rs: 11 lines
- lib.rs: ~350+ lines (state + view logic)
- actions.rs: ~20 lines
- avatars.rs: ~150+ lines (self-contained resource)
- components.rs: ~15 lines (exports only)
- html_content.rs: ~200+ lines (utility)
- login_flow.rs: ~50+ lines (feature-in-development)
- components/timeline.rs: ~100-300 lines (estimated)
- components/thread.rs: ~100-300 lines (estimated)

Total: ~1500-2000 lines in modular, organized files

---

## Implementation Order

1. **Read**: REFACTORING_SUMMARY.md completely (15 min)
2. **Understand**: XILEM_ARCHITECTURE.md sections 1-5 (30 min)
3. **Plan**: Use REFACTORING_SUMMARY.md's 6 phases as your plan
4. **Code Phase 1**: Reference XILEM_CODE_PATTERNS.md sections 1-2
5. **Code Phase 2**: Reference XILEM_CODE_PATTERNS.md sections 3-6
6. **Code Phase 3-5**: Reference XILEM_CODE_PATTERNS.md sections 7-11
7. **Verify**: Check against PLACEHERO_STRUCTURE.txt - Pattern Checklist

---

## Learning Resources

Inside this documentation:
- **XILEM_ARCHITECTURE.md** has section 13 with sidebar pattern for font editor
- **REFACTORING_SUMMARY.md** has Testing Strategy section
- **XILEM_CODE_PATTERNS.md** has Refactoring Template at the end

External resources:
- Xilem on GitHub: https://github.com/linebender/xilem
- Placehero example: https://github.com/linebender/xilem/tree/main/placehero
- Xilem's masonry layout: https://github.com/linebender/masonry-rs

---

## Common Questions Answered

**Q: Where do I put my state?**
A: In lib.rs, as struct definitions. See XILEM_CODE_PATTERNS.md section 2.

**Q: Where do I put my async code?**
A: In async_tasks/ modules. See XILEM_CODE_PATTERNS.md sections 7-8 and REFACTORING_SUMMARY.md Phase 5.

**Q: Where do I update state?**
A: In ONE place - the map_action closure in app_view. See REFACTORING_SUMMARY.md "Central Action Handling" or XILEM_CODE_PATTERNS.md section 4.

**Q: How do I handle multiple UI modes?**
A: Use MainState enum with variants. See XILEM_CODE_PATTERNS.md section 2.

**Q: How do I make one view show different content based on state?**
A: Use OneOf types for conditional rendering. See XILEM_CODE_PATTERNS.md section 10.

**Q: How do I share state across views?**
A: Either put it in main state struct, or use Resource providers for caching. See XILEM_CODE_PATTERNS.md section 9.

**Q: How long will refactoring take?**
A: 12-21 hours (1-3 days). See REFACTORING_SUMMARY.md "Estimated Effort".

---

## Success Metrics

After successful refactoring, you should have:

- [ ] main.rs with only 10-15 lines (entry point only)
- [ ] lib.rs with state definitions and root view logic
- [ ] actions.rs with single action enum
- [ ] Views organized by UI area (sidebar, canvas, etc.)
- [ ] Components in their own module
- [ ] Async operations in separate files
- [ ] All state updates in one map_action closure
- [ ] Can add new features without modifying main.rs
- [ ] Can test individual views independently

---

## File Statistics

This documentation package contains:

- **REFACTORING_SUMMARY.md**: 13 KB, 400+ lines
- **XILEM_ARCHITECTURE.md**: 17 KB, 700+ lines
- **XILEM_CODE_PATTERNS.md**: 23 KB, 900+ lines
- **PLACEHERO_STRUCTURE.txt**: 12 KB, 500+ lines

**Total**: 65 KB of documentation with 2500+ lines of content

All based on the actual Xilem Placehero example application from:
https://github.com/linebender/xilem/tree/main/placehero

---

## Getting Started Right Now

1. Open REFACTORING_SUMMARY.md
2. Read "Target Architecture" section (2 min)
3. Read "Key Patterns to Adopt" section (5 min)
4. Read "Refactoring Steps" - Phase 1 only (5 min)
5. Start implementing Phase 1
6. Reference XILEM_CODE_PATTERNS.md sections 1-2 while coding

That's it! You're ready to start refactoring.

---

**Last updated**: Based on Xilem Placehero as of latest commits
**Repository**: https://github.com/linebender/xilem
**Example**: https://github.com/linebender/xilem/tree/main/placehero
