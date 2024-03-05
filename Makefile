CC = g++

SRC = ./src/main.cpp
EXEC = main

build: $(SRC)
	$(CC) $? -o $(EXEC)

clean:
	rm -r main