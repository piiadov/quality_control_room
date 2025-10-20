#ifndef TEST_XGBWRAPPER_H
#define TEST_XGBWRAPPER_H

#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <math.h>
#include <string.h>
#include "../src/xgbwrapper.h"
#include <limits.h>
#include <xgboost/c_api.h>

// Function declarations
void test_shuffle();
void test_split_data();
void test_generate_data();
void test_generate_simple_data();
void test_xgboost();

void generate_data_2cols(float* x, float* y, int rows, int x_cols);
void generate_simple_data_2cols(float* x, float* y, int rows, int x_cols);
void print_data(float* x, float* y, int rows, int x_cols, int y_cols);

#endif // TEST_XGBWRAPPER_H