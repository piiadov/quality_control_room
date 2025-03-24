#ifndef XGBWRAPPER_H
#define XGBWRAPPER_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    const char *key;
    const char *value;
} KVPair;

void shuffle(int* array, int n);
void split_data(const float* x, const float* y,
    float* x_train, float* y_train, 
    float* x_test, float* y_test,
    int x_cols, int y_cols, int rows, int rows_train);
void calculate_rmse(const float* y_pred, const float* y_test,
    int rows, int y_cols, float* rmse);
void train(float* x, float* y, int rows, int x_cols, int y_cols, 
    const KVPair* config, int len_config, const char* inference_path);
void generate_data_2cols(float* x, float* y, int rows, int x_cols);
void predict(float* data, int rows, int x_cols, int y_cols,
    const char* inference_path, float* pred);
void print_rmse(float* rmse, int cols);

#ifdef __cplusplus
}
#endif

#endif // XGBWRAPPER_H
