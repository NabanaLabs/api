CREATE TABLE Logs (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    event_executer TEXT NOT NULL,
    message TEXT NOT NULL
);