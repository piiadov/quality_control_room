#include "xgbwrapper.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <math.h>
#include <cjson/cJSON.h>
#include </home/vp/GitHub/xgboost/include/xgboost/c_api.h>

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

void calculate_rmse(const float* y_pred, const float* y_test,
                    int rows, int y_cols, float *rmse) {
    
    for (int j = 0; j < y_cols; ++j) {
        float mse = 0.0;
        for (int i = 0; i < rows; ++i) {
            float diff = y_test[i * y_cols + j] - y_pred[i * y_cols + j];
            mse += diff * diff;
        }
        mse /= rows;
        rmse[j] = sqrt(mse);
    }
}

char* read_config(const char* filename) {
    FILE* file = fopen(filename, "rb");
    if (!file) {
        return NULL;
    }
    
    fseek(file, 0, SEEK_END);
    long length = ftell(file);
    fseek(file, 0, SEEK_SET);

    char* content = (char*)malloc(length + 1);
    if (content) {
        if (fread(content, 1, length, file) != length) {
            free(content);
            fclose(file);
            return NULL;
        }
        content[length] = '\0';
    }

    fclose(file);
    return content;
}

void xgb_train(float* x, float *y, int rows, int x_cols, int y_cols) {
    DMatrixHandle dtrain;
    XGDMatrixCreateFromMat(x, rows, x_cols, -1, &dtrain);
    XGDMatrixSetFloatInfo(dtrain, "label", y, rows * y_cols);

    // Set parameters for the booster
    BoosterHandle booster;
    XGBoosterCreate(&dtrain, 1, &booster);

    // Read config.json
    char* config = read_config(CONFIG_PATH);
    if (config == NULL) {
        fprintf(stderr, "Failed to read config file.\n");
        exit(EXIT_FAILURE);
    }
    // Parse the JSON config
    cJSON *json = cJSON_Parse(config);
    if (json == NULL) {
        fprintf(stderr, "Error parsing config file.\n");
        free(config);
        exit(EXIT_FAILURE);
    }

    // Set XGBoost parameters
    cJSON *xgb_params = cJSON_GetObjectItemCaseSensitive(json, "xgb_params");
    if (cJSON_IsObject(xgb_params)) {
        cJSON *param = NULL;
        cJSON_ArrayForEach(param, xgb_params) {
            if (cJSON_IsString(param)) {
                int status = XGBoosterSetParam(booster, param->string, param->valuestring);
                if (status != 0) {
                    fprintf(stderr, "Failed to set parameter %s\n", param->string);
                }
            }
        }
    }

    // Extract n_estimators value and cast to int
    cJSON *n_estimators_param = cJSON_GetObjectItemCaseSensitive(xgb_params, "n_estimators");
    
    if (n_estimators_param == NULL) {
        fprintf(stderr, "Error: n_estimators parameter is missing.\n");
        exit(EXIT_FAILURE);
    }
 
    int n_estimators = atoi(n_estimators_param->valuestring);      
    if (n_estimators < 1) {
        fprintf(stderr, "Error: n_estimators parameter has a value of 0 or less.\n");
        exit(EXIT_FAILURE);
    }

    // Clean up config and JSON
    cJSON_Delete(json);
    free(config);

    // Train the model
    for (int i = 0; i < n_estimators; ++i) {
        XGBoosterUpdateOneIter(booster, i, dtrain);
    }

    // Save the trained model to a file
    XGBoosterSaveModel(booster, INFERENCE_PATH);

    // Clean up XGBoost
    XGBoosterFree(booster);
    XGDMatrixFree(dtrain);
}

void xgb_predict(float* data, int rows, int cols, float* pred) {
    DMatrixHandle dmatrix;
    XGDMatrixCreateFromMat(data, rows, cols, -1, &dmatrix);

    BoosterHandle booster;
    XGBoosterCreate(NULL, 0, &booster);

    // Load the trained model from a file
    int status = XGBoosterLoadModel(booster, INFERENCE_PATH);
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

    // Copy the prediction results to the output array
    for (bst_ulong i = 0; i < out_len; ++i) {
        pred[i] = out_result[i];
    }

    // Clean up
    XGBoosterFree(booster);
    XGDMatrixFree(dmatrix);
}

void print_rmse(float* rmse, int cols) {
    printf("RMSE: ");
    for (int i = 0; i < cols; ++i) {
        printf("%f ", rmse[i]);
    }
    printf("\n");
}
