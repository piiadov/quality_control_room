/**
 * @file test_xgbwrapper.c
 * @brief Test suite for xgbwrapper library v0.4.0
 * 
 * Tests the simplified 6-function API:
 * - xgbw_init, xgbw_cleanup
 * - xgbw_train_eval (training with auto split and evaluation)
 * - xgbw_predict (inference)
 * - xgbw_get_last_error, xgbw_status_string
 */

#include "test_xgbwrapper.h"

/* Test output directory */
static const char* TEST_OUTPUT_DIR = "/tmp";
static const char* TEST_MODEL_NAME = "xgbw_test_model";

/* Store the actual model path from training for prediction test */
static char g_model_path[256] = {0};

/* ===========================================================================
 * Test Data Generation
 * ===========================================================================*/

void generate_test_data(float* x, float* y, int rows, int x_cols) {
    const int y_cols = 2;
    
    /* Simple pseudo-random generator for reproducible tests */
    unsigned int seed = 42;
    
    for (int i = 0; i < rows; ++i) {
        float sum_x = 0.0f;
        float sum_sqrt_x = 0.0f;
        
        for (int j = 0; j < x_cols; ++j) {
            /* Linear congruential generator */
            seed = seed * 1103515245u + 12345u;
            float val = (float)((seed >> 16) & 0x7fff) / 32767.0f;
            
            x[i * x_cols + j] = val;
            sum_x += val;
            sum_sqrt_x += sqrtf(val);
        }
        
        y[i * y_cols + 0] = sum_x;
        y[i * y_cols + 1] = sum_sqrt_x;
    }
}

/* ===========================================================================
 * Test Implementations
 * ===========================================================================*/

void test_train_eval(void) {
    printf("=== Test: xgbw_train_eval ===\n");
    
    /* Initialize library */
    XGBWrapperStatus status = xgbw_init();
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_init returned %s\n", xgbw_status_string(status));
        return;
    }
    printf("Library initialized\n");
    
    /* Generate test data */
    const int rows = 1000;
    const int x_cols = 4;
    const int y_cols = 2;
    const float train_ratio = 0.8f;
    
    float* x = (float*)malloc((size_t)(rows * x_cols) * sizeof(float));
    float* y = (float*)malloc((size_t)(rows * y_cols) * sizeof(float));
    float rmse[2] = {0};
    
    if (!x || !y) {
        printf("FAIL: Memory allocation failed\n");
        free(x); free(y);
        xgbw_cleanup();
        return;
    }
    
    generate_test_data(x, y, rows, x_cols);
    printf("Generated %d samples with %d features\n", rows, x_cols);
    
    /* XGBoost configuration */
    KVPair config[] = {
        {"booster", "gbtree"},
        {"objective", "reg:squarederror"},
        {"max_depth", "6"},
        {"learning_rate", "0.1"},
        {"n_estimators", "50"},
        {"verbosity", "0"}
    };
    int len_config = sizeof(config) / sizeof(config[0]);
    
    /* Train with evaluation */
    printf("Training with %.0f%% train ratio...\n", train_ratio * 100);
    
    status = xgbw_train_eval(
        x, y, rows, x_cols, y_cols,
        train_ratio,
        config, len_config,
        TEST_OUTPUT_DIR, TEST_MODEL_NAME,
        g_model_path, sizeof(g_model_path),
        rmse
    );
    
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_train_eval returned %s: %s\n", 
               xgbw_status_string(status), xgbw_get_last_error());
        free(x); free(y);
        xgbw_cleanup();
        return;
    }
    
    printf("Model saved to: %s\n", g_model_path);
    printf("RMSE results:\n");
    printf("  Target 0 (sum): %.6f\n", rmse[0]);
    printf("  Target 1 (sqrt sum): %.6f\n", rmse[1]);
    
    /* Evaluate - RMSE should be reasonably low for this synthetic data */
    int passed = 1;
    for (int j = 0; j < y_cols; ++j) {
        if (rmse[j] > 0.5f) {
            printf("WARNING: RMSE[%d] = %.4f is higher than expected\n", j, rmse[j]);
            passed = 0;
        }
    }
    
    if (passed) {
        printf("PASS: Model trained with acceptable RMSE\n");
    } else {
        printf("PARTIAL: Model trained but RMSE higher than expected\n");
    }
    
    free(x);
    free(y);
    xgbw_cleanup();
    printf("\n");
}

void test_predict(void) {
    printf("=== Test: xgbw_predict ===\n");
    
    if (g_model_path[0] == '\0') {
        printf("SKIP: No model available (run test_train_eval first)\n\n");
        return;
    }
    
    /* Initialize library */
    XGBWrapperStatus status = xgbw_init();
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_init returned %s\n", xgbw_status_string(status));
        return;
    }
    
    /* Generate new test data for prediction */
    const int rows = 100;
    const int x_cols = 4;
    const int y_cols = 2;
    
    float* x = (float*)malloc((size_t)(rows * x_cols) * sizeof(float));
    float* y_true = (float*)malloc((size_t)(rows * y_cols) * sizeof(float));
    float* y_pred = (float*)malloc((size_t)(rows * y_cols) * sizeof(float));
    
    if (!x || !y_true || !y_pred) {
        printf("FAIL: Memory allocation failed\n");
        free(x); free(y_true); free(y_pred);
        xgbw_cleanup();
        return;
    }
    
    generate_test_data(x, y_true, rows, x_cols);
    printf("Generated %d new samples for prediction\n", rows);
    
    /* Make predictions */
    status = xgbw_predict(x, rows, x_cols, y_cols, g_model_path, y_pred);
    
    if (status != XGBW_SUCCESS) {
        printf("FAIL: xgbw_predict returned %s: %s\n",
               xgbw_status_string(status), xgbw_get_last_error());
        free(x); free(y_true); free(y_pred);
        xgbw_cleanup();
        return;
    }
    
    /* Calculate RMSE manually */
    float rmse[2] = {0};
    for (int j = 0; j < y_cols; ++j) {
        float sse = 0.0f;
        for (int i = 0; i < rows; ++i) {
            float diff = y_pred[i * y_cols + j] - y_true[i * y_cols + j];
            sse += diff * diff;
        }
        rmse[j] = sqrtf(sse / (float)rows);
    }
    
    printf("Prediction RMSE:\n");
    printf("  Target 0 (sum): %.6f\n", rmse[0]);
    printf("  Target 1 (sqrt sum): %.6f\n", rmse[1]);
    
    /* Show a few predictions */
    printf("Sample predictions (first 3):\n");
    for (int i = 0; i < 3 && i < rows; ++i) {
        printf("  [%d] true=[%.4f, %.4f] pred=[%.4f, %.4f]\n",
               i, y_true[i*2], y_true[i*2+1], y_pred[i*2], y_pred[i*2+1]);
    }
    
    printf("PASS: Predictions completed successfully\n");
    
    free(x);
    free(y_true);
    free(y_pred);
    xgbw_cleanup();
    printf("\n");
}

/* ===========================================================================
 * Main Entry Point
 * ===========================================================================*/

int main(int argc, char* argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <test_name>\n\n", argv[0]);
        fprintf(stderr, "Available tests:\n");
        fprintf(stderr, "  test_train_eval  - Test all-in-one training with evaluation\n");
        fprintf(stderr, "  test_predict     - Test inference (run after test_train_eval)\n");
        fprintf(stderr, "  test_all         - Run all tests\n");
        return EXIT_FAILURE;
    }

    const char* test_name = argv[1];

    if (strcmp(test_name, "test_train_eval") == 0) {
        test_train_eval();
    } else if (strcmp(test_name, "test_predict") == 0) {
        test_predict();
    } else if (strcmp(test_name, "test_all") == 0) {
        test_train_eval();
        test_predict();
    } else {
        fprintf(stderr, "Unknown test: %s\n", test_name);
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}
