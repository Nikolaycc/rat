CC = gcc
CFLAGS = -DDEBUG -ggdb -Wall -Wextra -g -I./rat_lib -I./rat_cli -I./rat_rust
LDFLAGS = -L. -lrat

LIB_DIR = rat_lib
#CLI_DIR = rat_cli
#RUST_DIR = rat_rust
TEST_DIR = tests
OBJ_DIR = obj
BIN_DIR = bin

LIB = librat.a

LIB_SRC = $(wildcard $(LIB_DIR)/*.c)
LIB_OBJ = $(patsubst %.c, $(OBJ_DIR)/%.o, $(notdir $(LIB_SRC)))

TEST_SRC = $(wildcard $(TEST_DIR)/*.c)
TEST_BINS = $(patsubst $(TEST_DIR)/%.c, $(BIN_DIR)/%, $(TEST_SRC))
TEST_OBJ = $(patsubst $(TEST_DIR)/%.c, $(OBJ_DIR)/%.o, $(TEST_SRC))

all: $(LIB) $(TEST_BINS)

$(OBJ_DIR) $(BIN_DIR):
	mkdir -p $@

$(LIB): $(LIB_OBJ)
	ar rcs $@ $^

$(OBJ_DIR)/%.o: $(LIB_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJ_DIR)/%.o: $(CLI_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJ_DIR)/%.o: $(RUST_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJ_DIR)/%.o: $(TEST_DIR)/%.c | $(OBJ_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BIN_DIR)/%: $(OBJ_DIR)/%.o $(LIB) | $(BIN_DIR)
	$(CC) $< -o $@ $(LDFLAGS)

clean:
	rm -rf $(OBJ_DIR) $(BIN_DIR) $(LIB)

.PHONY: all clean
