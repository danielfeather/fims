CREATE TABLE
    expenses (
        id uuid NOT NULL CONSTRAINT expenses_pk PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        currency CHAR(3) NOT NULL,
        amount INTEGER NOT NULL,
        frequency VARCHAR(8) NOT NULL,
        start_date DATE NOT NULL,
        end_date DATE
    );