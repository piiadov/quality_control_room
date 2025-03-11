#include "test_xgbwrapper.h"

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

void generate_simple_data_2cols(float* x, float* y, int rows, int x_cols) {
    // Generate x as sequences of numbers and y1 as sum(x) and y2 as -sum(x)
    for (int i = 0; i < rows; ++i) {
        for (int j = 0; j < x_cols; ++j) {
            x[i * x_cols + j] = i * x_cols + j;
        }
    }

    const int y_cols = 2;
    for (int i = 0; i < rows; ++i) {  
        y[i * y_cols] = 0;
        for (int k = 0; k < x_cols; ++k) {
            y[i * y_cols] += x[i * x_cols + k];
        }
        y[i * y_cols + 1] = 0;
        for (int k = 0; k < x_cols; ++k) {
            y[i * y_cols + 1] -= x[i * x_cols + k];
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

void test_shuffle() {
    int n = 10;
    int* array = (int*)malloc(n * sizeof(int));
    for (int i = 0; i < n; ++i) {
        array[i] = i;
    }
    printf("Original array:\n");
    for (int i = 0; i < n; ++i) {
        printf("%d ", array[i]);
    }
    printf("\n");
    shuffle(array, n);
    printf("Print shuffled array:\n"); 
    for (int i = 0; i < n; ++i) {
        printf("%d ", array[i]);
    }
    printf("\n");
    free(array);
}

void test_split_data() {
    int rows = 10;
    int x_cols = 2;
    int y_cols = 1;
    float* x = (float*)malloc(rows * x_cols * sizeof(float));
    float* y = (float*)malloc(rows * y_cols * sizeof(float));
    for (int i = 0; i < rows; ++i) {
        for (int j = 0; j < x_cols; ++j) {
            x[i * x_cols + j] = i * x_cols + j;
        }
        y[i * y_cols] = i;
    }
    float* x_train = (float*)malloc(8 * x_cols * sizeof(float));
    float* y_train = (float*)malloc(8 * y_cols * sizeof(float));
    float* x_test = (float*)malloc(2 * x_cols * sizeof(float));
    float* y_test = (float*)malloc(2 * y_cols * sizeof(float));
    split_data(x, y, x_train, y_train, x_test, y_test, x_cols, y_cols, rows, 8);
    printf("x_train:\n");
    for (int i = 0; i < 8; ++i) {
        for (int j = 0; j < x_cols; ++j) {
            printf("%f ", x_train[i * x_cols + j]);
        }
        printf("\n");
    }
    printf("y_train:\n");
    for (int i = 0; i < 8; ++i) {
        for (int j = 0; j < y_cols; ++j) {
            printf("%f ", y_train[i * y_cols + j]);
        }
        printf("\n");
    }
    printf("x_test:\n");
    for (int i = 0; i < 2; ++i) {
        for (int j = 0; j < x_cols; ++j) {
            printf("%f ", x_test[i * x_cols + j]);
        }
        printf("\n");
    }
    printf("y_test:\n");
    for (int i = 0; i < 2; ++i) {
        for (int j = 0; j < y_cols; ++j) {
            printf("%f ", y_test[i * y_cols + j]);
        }
        printf("\n");
    }
    free(x);
    free(y);
    free(x_train);
    free(y_train);
    free(x_test);
    free(y_test);
}

void test_generate_data() {
    int rows = 10;
    int x_cols = 2;
    const int y_cols = 2;
    float* x = (float*)malloc(rows * x_cols * sizeof(float));
    float* y = (float*)malloc(rows * y_cols * sizeof(float));
    generate_data_2cols(x, y, rows, x_cols);
    print_data(x, y, rows, x_cols, y_cols);
    free(x);
    free(y);
}

void test_generate_simple_data() {
    int rows = 10;
    int x_cols = 2;
    const int y_cols = 2;
    float* x = (float*)malloc(rows * x_cols * sizeof(float));
    float* y = (float*)malloc(rows * y_cols * sizeof(float));
    generate_simple_data_2cols(x, y, rows, x_cols);
    print_data(x, y, rows, x_cols, y_cols);
    free(x);
    free(y);
}


void test_xgboost() {
    // Test for 10000 rows, 4 features, 2 outputs
    // with data from generate_data_2cols()

    // Generate test data
    int rows = 10000;
    int x_cols = 4; // 4 features
    const int y_cols = 2; // 2 outputs
    const float split_ratio = 0.8; // 80% for training

    float* x = (float*)malloc(rows * x_cols * sizeof(float));
    float* y = (float*)malloc(rows * y_cols * sizeof(float));
    generate_data_2cols(x, y, rows, x_cols);

    // Allocate memory for train and test sets
    int rows_train = (int) (rows * split_ratio); 
    int rows_test = rows - rows_train;
    float* x_train = (float*)malloc(rows_train * x_cols * sizeof(float));
    float* y_train = (float*)malloc(rows_train * y_cols * sizeof(float));
    float* x_test = (float*)malloc(rows_test * x_cols * sizeof(float));
    float* y_test = (float*)malloc(rows_test * y_cols * sizeof(float));

    // Split data into train and test sets
    split_data(x, y, x_train, y_train, x_test, y_test, x_cols, y_cols, rows, rows_train);
    
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
    print_rsme(rmse, y_cols);

    free(y_test);
    free(y_pred);
    free(rmse);
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <test_function>\n", argv[0]);
        return EXIT_FAILURE;
    }

    if (strcmp(argv[1], "test_shuffle") == 0) {
        test_shuffle();
    } else if (strcmp(argv[1], "test_split_data") == 0) {
        test_split_data();
    } else if (strcmp(argv[1], "test_generate_data") == 0) {
        test_generate_data();
    } else if (strcmp(argv[1], "test_generate_simple_data") == 0) {
        test_generate_simple_data();
    } else if (strcmp(argv[1], "test_xgboost") == 0) {
        test_xgboost();
    } else {
        fprintf(stderr, "Unknown test function: %s\n", argv[1]);
        return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}
