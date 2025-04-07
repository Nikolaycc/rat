CC = gcc
CFLAGS = -ggdb -Wall -Wextra -g -I./src
LDFLAGS = -L. -lrat

SRC_DIR = src
TEST_DIR = tests
OBJ_DIR = obj
BIN_DIR = bin

LIB = librat.a

LIB_SRC = $(wildcard $(SRC_DIR)/*.c)
LIB_OBJ = $(patsubst $(SRC_DIR)/%.c, $(OBJ_DIR)/%.o, $(LIB_SRC))

TEST_SRC = $(wildcard $(TEST_DIR)/*.c)
TEST_BINS = $(patsubst $(TEST_DIR)/%.c, $(BIN_DIR)/%, $(TEST_SRC))
TEST_OBJ = $(patsubst $(TEST_DIR)/%.c, $(OBJ_DIR)/%.o, $(TEST_SRC))

all: $(LIB) $(TEST_BINS)

$(OBJ_DIR) $(BIN_DIR):
	mkdir -p $@

$(LIB): $(LIB_OBJ)
	ar rcs $@ $^

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJ_DIR)/%.o: $(TEST_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BIN_DIR)/%: $(OBJ_DIR)/%.o $(LIB) | $(BIN_DIR)
	$(CC) $< -o $@ $(LDFLAGS)

clean:
	rm -rf $(OBJ_DIR) $(BIN_DIR) $(LIB)

.PHONY: all clean
