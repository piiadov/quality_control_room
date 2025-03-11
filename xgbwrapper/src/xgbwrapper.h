#ifndef XGBWRAPPER_H
#define XGBWRAPPER_H

void shuffle(int *array, int n);
void split_data(const float* x, const float* y,
    float* x_train, float* y_train, 
    float* x_test, float* y_test,
    int x_cols, int y_cols, int rows, int rows_train);
void calculate_rmse(const float* y_pred, const float* y_test,
    int rows, int y_cols, float* rmse);
void xgb_train(float* x, float* y, int rows, int x_cols, int y_cols);
void xgb_predict(float* data, int rows, int cols, float* pred);
void print_rsme(float* rmse, int cols);

#ifdef __cplusplus
extern "C" {
#endif

double my_function(double x);

#ifdef __cplusplus
}
#endif

#endif // XGBWRAPPER_H
