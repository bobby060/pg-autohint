# AutoHint Design Doc

## Overview

>What motivates this to be implemented? What will this component achieve? 
The main purpose of this project is to create an automatic method of identifying poor Postgres query plans *without modifying the dbms* and to add hints to the sql that rectify those poor plans.  

The secondary purpose of this project is to conduct some preliminary work to enable the creation of an Optd to SQL adapter next semester. This project allows us to explore two problems directly transferable to that work:
1. Practice converting a physical plan to SQL via an abstract syntax tree
2. Experiment with identifying hints required to preserve the semantic meaning of a physical plan in SQL

## Scope
>Which parts of the system will this feature rely on or modify? Write down specifics so people involved can review the design doc

## Glossary (Optional)

>If you are introducing new concepts or giving unintuitive names to components, write them down here.

## Architectural Design
>Explain the input and output of the component, describe interactions and breakdown the smaller components if any. Include diagrams if appropriate.

TODO: put this into a graph
- Connector: Connects to a postgres DB and retreives query plan (either with EXPLAIN or EXPLAIN ANALYZE) to a serialized `PlanNode` struct
- Optimizer: Applies a list of rules to a `PlanNode` and outputs the list of hints prepended to original sql
- Postgres Plan: Struct representation of a Postgres Plan
- Plan2ast: Converter that takes a `PlanRoot` as input and outputs the equivalent Datafustion Abstract Syntax Tree (incomplete and proof of concept)
    - TODO: Outline what works in plan2ast




## Design Rationale
>Explain the goals of this design and how the design achieves these goals. Present alternatives considered and document why they are not chosen.
Current design: process plan and provide a list of hints that can be prepending to original sql. In this design, we traverse the Postgres plan, but do not have to convert that plan back into an AST.

Alternative design considered: converting the physical plan to a Datafusion AST, then applying simple heuristic rules that not only provide a list of hints, but also allow manipulation of the underlying SQL. This would allow things like converting redundant filters (e.g. `SELECT X + 0 FROM A` to  `SELECT X FROM A`). However, we quickly identified that this conversion requires a signifant amount of work to convert correctly. While we able to relatively easily implement converting most SPJ queries, two specific outliers gave us problems: subqueries and correct output columns for ORDER BY. We have preserved our work on this and retained the testing infrstructure and may continue work in the future in addition to reusing the code on Optd adapters in the future.



## Testing Plan
>How should the component be tested?
- Component level unit testing
- End to end benchmarks (?)

## Trade-offs and Potential Problems
>Write down any conscious trade-off you made that can be problematic in the future, or any problems discovered during the design process that remain unaddressed (technical debts).
- Talk about difficulty of creating rules to optimize. This is what we will focus on the most for the rest of the semester

## Future Work
>Write down future work to fix known problems or otherwise improve the component.
- Add more rules
- Continue implementation of plan2ast converter
- Complete documentation to make this framework very accessible

