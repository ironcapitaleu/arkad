```mermaid
---
title: "SEC Data Processing Pipeline: State Machine Overview"
---
stateDiagram-v2
    %% Align all the SuperStates from left-to-right
    direction LR

    %% `Extract` SuperState
    [*] --> Extract
    state Extract {
        ValidateCikFormat --> PrepareSecRequest
        PrepareSecRequest --> ExecuteSecRequest
        ExecuteSecRequest --> StoreCompanyFacts
        note right of StoreCompanyFacts : No binary runs this state
    }

    Extract --> Transform

    %% `Transform` SuperState
    state Transform {
        ParseCompanyFacts --> CreateFinancialStatements
    }

    Transform --> Load

    %% `Load` SuperState
    state Load {
        StoreData
    }

    Load --> [*]
```
