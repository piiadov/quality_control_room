/**
 * @file test_xgbwrapper.h
 * @brief Test suite for xgbwrapper library v0.4.0
 */

#ifndef TEST_XGBWRAPPER_H
#define TEST_XGBWRAPPER_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

#include "xgbwrapper.h"

/**
 * @brief Test xgbw_train_eval (all-in-one training with evaluation)
 */
void test_train_eval(void);

/**
 * @brief Test xgbw_predict (inference on saved model)
 */
void test_predict(void);

/**
 * @brief Generate synthetic test data with known relationships
 * @param x Output features (rows * x_cols)
 * @param y Output targets (rows * y_cols)
 * @param rows Number of samples
 * @param x_cols Number of features
 * 
 * y[0] = sum(x), y[1] = sum(sqrt(x))
 */
void generate_test_data(float* x, float* y, int rows, int x_cols);

#endif /* TEST_XGBWRAPPER_H */
