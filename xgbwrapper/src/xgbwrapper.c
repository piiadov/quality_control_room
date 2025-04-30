#include "xgbwrapper.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <math.h>
#include <string.h>
#include </home/vp/xgboost/include/xgboost/c_api.h>

void generate_data_2cols(float* x, float* y, int rows, int x_cols) {
    // Generate random x values and calculate y values
    srand(time(NULL));
    for (int i = 0; i < rows; ++i) {
        for (int j = 0; j < x_cols; ++j) {
            x[i * x_cols + j] = (float) rand() / RAND_MAX;
        }
    }

    // y_cols must be 2 for the test function: y1 = sum(x) and y2 = sum(sqrt(x))
    const int y_cols = 2; 
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

void shuffle(int *array, int n) {
    for (int i = 0; i < n; ++i) {
        array[i] = i;
    }
    srand(time(NULL));
    if (n > 1) {
        size_t i;
        for (i = 0; i < n - 1; i++) {
            size_t j = i + rand() / (RAND_MAX / (n - i) + 1);
            int t = array[j];
            array[j] = array[i];
            array[i] = t;
        }
    }
}

void split_data(const float* x, const float* y,
                float* x_train, float* y_train, 
                float* x_test, float* y_test,
                int x_cols, int y_cols, int rows, int rows_train) {

    int* indices = (int*)malloc(rows * sizeof(int));
    shuffle(indices, (size_t)rows);

    for (int i = 0; i < rows_train; ++i) {
        int x_train_idx = i * x_cols;
        int x_idx = indices[i] * x_cols;
        for (int j = 0; j < x_cols; ++j) {
            x_train[x_train_idx + j] = x[x_idx + j];
        }
        int y_train_idx = i * y_cols;
        int y_idx = indices[i] * y_cols;
        for (int j = 0; j < y_cols; ++j) {
            y_train[y_train_idx + j] = y[y_idx + j];
        }
    }

    int rows_test = rows - rows_train;
    for (int i = 0; i < rows_test; ++i) {
        int x_test_idx = i * x_cols;
        int x_idx = indices[rows_train + i] * x_cols;
        for (int j = 0; j < x_cols; ++j) {
            x_test[x_test_idx + j] = x[x_idx + j];
        }
        int y_test_idx = i * y_cols;
        int y_idx = indices[rows_train + i] * y_cols;
        for (int j = 0; j < y_cols; ++j) {
            y_test[y_test_idx + j] = y[y_idx + j];
        }
    }
    free(indices);
}

void train(float* x, float *y, int rows, int x_cols, int y_cols, 
           const KVPair* config, int len_config, const char* inference_path) { 

    DMatrixHandle dtrain;
    XGDMatrixCreateFromMat(x, rows, x_cols, -1, &dtrain);
    XGDMatrixSetFloatInfo(dtrain, "label", y, rows * y_cols);

    // Set parameters for the booster
    BoosterHandle booster;
    XGBoosterCreate(&dtrain, 1, &booster);

    // Set XGBoost parameters
    int n_estimators = 0;
    for (int i = 0; i < len_config; ++i) {
        if (strcmp(config[i].key, "n_estimators") == 0) {
            n_estimators = atoi(config[i].value);
        }
        int status = XGBoosterSetParam(booster, config[i].key, config[i].value);
        if (status != 0) {
            fprintf(stderr, "Failed to set parameter %s: %s\n", config[i].key, config[i].value);
            exit(EXIT_FAILURE);
        }
    }
    if (n_estimators < 1) {
        fprintf(stderr, "Error: n_estimators parameter is missing or less than 1.\n");
        exit(EXIT_FAILURE);
    }

    // Train the model
    for (int i = 0; i < n_estimators; ++i) {
        XGBoosterUpdateOneIter(booster, i, dtrain);
    }

    XGBoosterSaveModel(booster, inference_path);
    XGBoosterFree(booster);
    XGDMatrixFree(dtrain);
}

void predict(float* data, int rows, int x_cols, int y_cols, const char* inference_path, 
             float* pred) {
    DMatrixHandle dmatrix;
    XGDMatrixCreateFromMat(data, rows, x_cols, -1, &dmatrix);

    BoosterHandle booster;
    XGBoosterCreate(NULL, 0, &booster);

    // Load the trained model from a file
    int status = XGBoosterLoadModel(booster, inference_path);
    if (status != 0) {
        fprintf(stderr, "Failed to load model from file.\n");
        XGBoosterFree(booster);
        XGDMatrixFree(dmatrix);
        exit(EXIT_FAILURE);
    }

    // Perform prediction
    bst_ulong out_len;
    const float* out_result;
    XGBoosterPredict(booster, dmatrix, 0, 0, 0, &out_len, &out_result);

    if (out_len != y_cols * rows) {
        fprintf(stderr, "Error: The number of predictions does not match the expected output size.\n");
        XGBoosterFree(booster);
        XGDMatrixFree(dmatrix);
        exit(EXIT_FAILURE);
    }

    // Copy the prediction results to the output array
    for (bst_ulong i = 0; i < out_len; ++i) {
        pred[i] = out_result[i];
    }

    // Clean up
    XGBoosterFree(booster);
    XGDMatrixFree(dmatrix);
}

void calculate_rmse(const float* y_pred, const float* y_test,
                    int rows, int y_cols, float *rmse) {
    for (int j = 0; j < y_cols; ++j) {
        float sse = 0.0;
        for (int i = 0; i < rows; ++i) {
            float diff = y_test[i * y_cols + j] - y_pred[i * y_cols + j];
            sse += diff * diff;
        }
        rmse[j] = sqrt(sse / rows);
    }
}

void print_rmse(float* rmse, int cols) {
    printf("RMSE: ");
    for (int i = 0; i < cols; ++i) {
        printf("%f ", rmse[i]);
    }
    printf("\n");
}