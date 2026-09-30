-- Execute the statement under the cursor with F5, or the whole file with F6.
SELECT user AS current_user, sysdate AS server_time FROM dual;

SELECT table_name, num_rows
FROM user_tables
ORDER BY table_name;

-- PL/SQL uses a slash on its own line as a block terminator.
BEGIN
    NULL;
END;
/
