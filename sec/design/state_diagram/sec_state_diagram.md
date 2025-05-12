```mermaid
---
title: Sec State Machine Implementation for Extracting, Processing, and Storing US Company Data
---
stateDiagram-v2
    [*] --> Extract
    state Extract {
        ValidateCikFormat --> PrepareSecRequest
        PrepareSecRequest --> ExecuteSecRequest
    }

    Extract --> Transform
    state Transform {
        ProcessSecData --> PrepareDataForStorage
    }
    
    Transform --> Load
    state Load {
        StoreData
    }

    Load --> [*]
```