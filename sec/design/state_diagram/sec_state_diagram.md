```mermaid
---
title: SEC Data Processing Pipeline: State Machine Overview
---
stateDiagram-v2
    %% Align all the SuperStates from left-to-right
    direction LR

    %% `Extract` SuperState
    [*] --> Extract
    state Extract {
        ValidateCikFormat --> PrepareSecRequest
        PrepareSecRequest --> ExecuteSecRequest
    }

    Extract --> Transform

    %% `Transform` SuperState
    state Transform {
        ProcessSecData --> PrepareDataForStorage
    }
    
    Transform --> Load

    %% `Load` SuperState
    state Load {
        StoreData
    }

    Load --> [*]
```