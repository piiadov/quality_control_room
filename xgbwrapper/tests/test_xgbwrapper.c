#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <math.h>
#include "../src/xgbwrapper.h"
#include <limits.h>
#include </home/vp/GitHub/xgboost/include/xgboost/c_api.h>

void gen_test_data_1(float* x, float* y, int rows, int x_cols, int y_cols) {
    // Generate random x values and calculate y values
    srand(time(NULL));
    for (int i = 0; i < rows; ++i) {
        for (int j = 0; j < x_cols; ++j) {
            x[i * x_cols + j] = (float) rand() / RAND_MAX;
        }
    }

    // y_cols must be 2 for the test function: y1 = sum(x) and y2 = sum(sqrt(x))
    for (int i = 0; i < rows; ++i) {  
        y[i * y_cols] = 0;
        for (int k = 0; k < x_cols; ++k) {
            y[i * y_cols] += x[i * x_cols + k];
        }
        y[i * y_cols + 1] = 0;
        for (int k = 0; k < x_cols; ++k) {
            y[i * y_cols + 1] += sqrt(x[i * x_cols + k]);
        }
    }
}

void print_data(float* x, float* y, int rows, int x_cols, int y_cols) {
    printf("x:\n");
    for (int i = 0; i < rows; ++i) {
        for (int j = 0; j < x_cols; ++j) {
            printf("%f ", x[i * x_cols + j]);
        }
        printf("\n");
    }
    printf("y:\n");
    for (int i = 0; i < rows; ++i) {
        for (int j = 0; j < y_cols; ++j) {
            printf("%f ", y[i * y_cols + j]);
        }
        printf("\n");
    }
}

int main() {
    
    // printf("Max int value (size of data array): %d\n", INT_MAX);
    // printf("Max unsigned long value: %lu\n", ULONG_MAX);

    // Generate test data
    int rows = 10000;
    int x_cols = 4;
    const int y_cols = 2;

    float* x = (float*)malloc(rows * x_cols * sizeof(float));
    float* y = (float*)malloc(rows * y_cols * sizeof(float));
    gen_test_data_1(x, y, rows, x_cols, y_cols);
    // print_data(x, y, rows, x_cols, y_cols);

    // Allocate memory for train and test sets
    int rows_train = (int) (rows * 0.8); // 80% for training
    int rows_test = rows - rows_train; // 20% for testing
    float* x_train = (float*)malloc(rows_train * x_cols * sizeof(float));
    float* y_train = (float*)malloc(rows_train * y_cols * sizeof(float));
    float* x_test = (float*)malloc(rows_test * x_cols * sizeof(float));
    float* y_test = (float*)malloc(rows_test * y_cols * sizeof(float));

    // Split data into train and test sets
    split_data(x, y, x_train, y_train, x_test, y_test, x_cols, y_cols, rows, rows_train);
    // print_data(x_train, y_train, rows_train, x_cols, y_cols);
    // print_data(x_test, y_test, rows_test, x_cols, y_cols);
    
    free(x);
    free(y);

    xgb_train(x_train, y_train, rows_train, x_cols, y_cols);
    free(x_train);
    free(y_train);

    float* y_pred = (float*)malloc(rows_test * y_cols * sizeof(float));
    xgb_predict(x_test, rows_test, x_cols, y_pred);
    free(x_test);

    float* rmse = (float*)malloc(y_cols * sizeof(float));
    calculate_rmse(y_pred, y_test, rows_test, y_cols, rmse);
    printf("RMSE: %f %f\n", rmse[0], rmse[1]);

    free(y_test);
    free(y_pred);
    free(rmse);

/*
    // Create DMatrix from the training data
    DMatrixHandle dtrain;
    XGDMatrixCreateFromMat(x_train, rows_train, x_cols, -1, &dtrain);
    XGDMatrixSetFloatInfo(dtrain, "label", y_train, rows_train * y_cols);

    // Set parameters for the booster
    BoosterHandle booster;
    XGBoosterCreate(&dtrain, 1, &booster);
    XGBoosterSetParam(booster, "booster", "gbtree");
    XGBoosterSetParam(booster, "n_thread", "8");
    XGBoosterSetParam(booster, "subsample", "1.0");
    XGBoosterSetParam(booster, "eval_metric", "rmse");
    XGBoosterSetParam(booster, "reg_alpha", "0.0");
    XGBoosterSetParam(booster, "reg_lambda", "0.0");
    XGBoosterSetParam(booster, "objective", "reg:squarederror");
    XGBoosterSetParam(booster, "max_depth", "10");
    XGBoosterSetParam(booster, "gamma", "0.0");
    XGBoosterSetParam(booster, "learning_rate", "0.3");
    XGBoosterSetParam(booster, "colsample_bytree", "0.3");
    XGBoosterSetParam(booster, "eta", "0.1");

    // Train the model
    for (int iter = 0; iter < 500; ++iter) {
        XGBoosterUpdateOneIter(booster, iter, dtrain);
    }

    // Save the trained model to a file
    XGBoosterSaveModel(booster, INFERENCE_PATH);

    // Create DMatrix from the test data
    DMatrixHandle dtest;
    XGDMatrixCreateFromMat(x_test, rows_test, x_cols, -1, &dtest);
    XGDMatrixSetFloatInfo(dtest, "label", y_test, rows_test * y_cols);

    // Make predictions
    bst_ulong out_len;
    const float* y_pred;
    XGBoosterPredict(booster, dtest, 0, 0, 0, &out_len, &y_pred);

    // Calculate mean squared error
    float* rmse = (float*)malloc(y_cols * sizeof(float));

    calculate_rmse(y_pred, y_test, rows_test, y_cols, rmse);
    printf("RMSE: %f %f\n", rmse[0], rmse[1]);

    XGBoosterFree(booster);
    XGDMatrixFree(dtrain);
    XGDMatrixFree(dtest);

    free(x_train);
    free(y_train);
    free(x_test);
    free(y_test);
    free(rmse);

*/


    double result = my_function(5.0);
    printf("Result: %f\n", result);

    


    return 0;
}
