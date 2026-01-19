/**
 * @file test_xgbwrapper.h
 * @brief Test suite for xgbwrapper library
 */

#ifndef TEST_XGBWRAPPER_H
#define TEST_XGBWRAPPER_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <math.h>

#include "xgbwrapper.h"

/* ===========================================================================
 * Test Function Declarations
 * ===========================================================================*/

/**
 * @brief Test the shuffle function
 */
void test_shuffle(void);

/**
 * @brief Test train/test data splitting
 */
void test_split_data(void);

/**
 * @brief Test random data generation
 */
void test_generate_data(void);

/**
 * @brief Test deterministic data generation
 */
void test_generate_simple_data(void);

/**
 * @brief End-to-end XGBoost training and prediction test
 */
void test_xgboost(void);

/* ===========================================================================
 * Test Utilities
 * ===========================================================================*/

/**
 * @brief Generate deterministic test data
 * 
 * Creates sequential features and computes:
 *   y[0] = sum(x)
 *   y[1] = -sum(x)
 */
void generate_simple_data_2cols(float* x, float* y, int rows, int x_cols);

/**
 * @brief Print data matrices for debugging
 */
void print_data(float* x, float* y, int rows, int x_cols, int y_cols);

#endif /* TEST_XGBWRAPPER_H */