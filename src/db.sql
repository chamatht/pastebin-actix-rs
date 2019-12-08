CREATE OR REPLACE FUNCTION pseudo_encrypt(VALUE BIGINT) returns int AS $$
DECLARE
l1 int;
l2 int;
r1 bigint;
r2 bigint;
i int:=0;
BEGIN
 l1:= (VALUE >> 16) & 65535;
 r1:= VALUE & 65535;
 WHILE i < 3 LOOP
   l2 := r1;
   r2 := l1 # ((((1366 * r1 + 150889) % 714025) / 714025.0) * 32767)::bigint;
   l1 := l2;
   r1 := r2;
   i := i + 1;
 END LOOP;
 RETURN ((r1 << 16) + l1);
END;
$$ LANGUAGE plpgsql strict immutable;

CREATE SEQUENCE uid_seq;

Create TABLE pastes (
    uid BIGINT NOT NULL UNIQUE PRIMARY KEY DEFAULT pseudo_encrypt(NEXTVAL('uid_seq')),
    title VARCHAR(101) NOT NULL,
    text VARCHAR(701)
);

INSERT INTO pastes(title, text) VALUES('Sample', 'This is a sample text.');
