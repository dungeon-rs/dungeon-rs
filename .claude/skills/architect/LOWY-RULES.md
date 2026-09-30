# Löwy's volatility-based decomposition: rule catalogue

From Juval Löwy, *Righting Software* (2019), and the IDesign method. Verify against the book when a rule is disputed; this catalogue is a working summary.

## Principles

- **Decompose by volatility, not by functionality.** Each component encapsulates an area of likely change. Functional decomposition yields either an explosion of small components or a few huge ones.
- **Never design against the requirements.** When requirements change, a volatility-based design does not; a functional one does.
- **Core use cases.** Usually 2–6; every other use case is a variation of one. Validate the design by drawing each as a call chain through the components, and look for symmetry.
- **Two axes of volatility.** The same customer over time, and different customers at the same time.
- **Volatility is not variability.** Open-ended change needs a component; bounded variation is a conditional.
- **Surface hidden volatility** by restating solutions as requirements ("send email" → "notify") and by comparing competitors.

## Component types

| Type | Encapsulates | Naming |
|---|---|---|
| **Client** | How the system is presented and used | by what it presents |
| **Manager** | Volatility in the sequence of a use case (the workflow) | `XxxManager` |
| **Engine** | Volatility in an activity or business rule | `XxxEngine` |
| **ResourceAccess** | Volatility in storage or integration; exposes atomic business verbs, never create/read/update/delete | `XxxAccess` |
| **Resource** | The storage or external system itself | – |
| **Utility** | Cross-cutting capability usable by any system ("could a cappuccino machine use it?") | by capability |

Volatility decreases top-down; reuse increases top-down.

**Sizing, as a smell test** (a count outside the range is not an error; justify it to the user by the distinct volatilities involved): about 10 components; 2–5 Managers, 2–3 Engines, 3–8 ResourceAccess and Resources, a handful of Utilities. Eight Managers means functional decomposition.

## Communication rules

- **Closed architecture**: a component calls only the layer directly beneath it, plus Utilities.
- Clients call Managers; a Client calls at most one Manager per use case.
- Managers call Engines and ResourceAccess. Managers call other Managers only asynchronously, through a queue.
- Engines call ResourceAccess. Engines never call other Engines, and never publish or subscribe to events.
- ResourceAccess calls Resources. ResourceAccess never calls another ResourceAccess.
- Calls to Engines and ResourceAccess are never queued.
- All layers may call Utilities.

## Contracts

- Business verbs, not data manipulation.
- 3–5 operations per contract; never more than about 20. A contract with one operation or with dozens is poorly factored.
