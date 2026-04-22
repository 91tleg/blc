# DB

## Events table

PK: EVENT#{event_id} — one item per event  
A separate counter item per event: PK EVENT#{event_id}, SK #COUNT — atomically incremented on registration, read for registered_count  
GSI not needed — list scans the table with a filter, small dataset for a school club  

## Registrations table

PK: EVENT#{event_id}, SK: REG#{registration_id} — all registrations for an event in one partition, enables efficient list + cursor pagination  
GSI: PK EVENT#{event_id}, SK EMAIL#{email} — enables the duplicate-email check without a scan  