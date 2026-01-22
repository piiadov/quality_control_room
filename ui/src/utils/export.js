import JSZip from "jszip";

/**
 * Export analysis results to a ZIP file
 * @param {Object} state - Store state object
 */
export async function exportResultsToZip(state) {
    const {
        samplingData,
        populationSize,
        sampleSize,
        minValue,
        maxValue,
        binsNumber,
        paramsMin,
        paramsMax,
        predictedParams,
        samplingParams,
        testMode,
        testModeParams,
        chi2Pred,
        chi2Min,
        chi2Max,
        binEdges,
        observedFreq,
        expectedFreqMin,
        expectedFreqMax,
        expectedFreqPred,
        scaledData,
        cdfMin,
        cdfMax,
        domain,
        fittedCdfMin,
        fittedCdfMax,
        fittedPdfMin,
        fittedPdfMax,
        predictedCdf,
        predictedPdf,
        samplingCdf,
        samplingPdf,
        testModeCdf,
        testModePdf,
    } = state;

    // Raw data CSV
    const csvData = [
        "Index,Value",
        ...samplingData.map((value, index) => `${index},${value}`)
    ].join("\n");

    // Result info CSV
    const resultInfo = [
        ["Population Size", populationSize],
        ["Sample Size", sampleSize],
        ["Min Value", minValue],
        ["Max Value", maxValue],
        ["Bins Number", binsNumber],
        ["Min Quality Parameters", paramsMin],
        ["Max Quality Parameters", paramsMax],
        ["Predicted Parameters", predictedParams],
        ["Sampling Parameters (MoM)", samplingParams],
        ...(testMode ? [["Test Mode Parameters", testModeParams]] : []),
        ["Critical Value", chi2Pred.critical_value],
        ["Predicted Chi2", chi2Pred.chi2],
        ["Predicted P-value", chi2Pred.p_value],
        ["Predicted Decision", chi2Pred.reject_null ? 'Reject' : 'Accept'],
        ["Min Quality Chi2", chi2Min.chi2],
        ["Min Quality P-value", chi2Min.p_value],
        ["Min Quality Decision", chi2Min.reject_null ? 'Reject' : 'Accept'],
        ["Max Quality Chi2", chi2Max.chi2],
        ["Max Quality P-value", chi2Max.p_value],
        ["Max Quality Decision", chi2Max.reject_null ? 'Reject' : 'Accept'],
    ];
    const csvResultInfo = resultInfo.map(([key, value]) => `${key},${value}`).join("\n");

    // Histogram CSV
    const csvFreqHist = [
        "Bin,Observed,Expected Min,Expected Max,Expected Predicted",
        ...binEdges.slice(0, -1).map((bin, i) => 
            `${bin},${observedFreq[i] ?? ''},${expectedFreqMin[i] ?? ''},${expectedFreqMax[i] ?? ''},${expectedFreqPred[i] ?? ''}`
        )
    ].join("\n");

    // Estimated CDF CSV
    const csvEstimatedCdf = [
        "Index,Scaled Data,CDF Min Quality,CDF Max Quality",
        ...scaledData.map((value, i) => `${i},${value},${cdfMin[i]},${cdfMax[i]}`)
    ].join("\n");

    // Full result data CSV
    const header = "x,CDF Min,CDF Max,PDF Min,PDF Max,CDF Predicted,PDF Predicted,CDF Sampling,PDF Sampling" +
        (testMode ? ",CDF TestMode,PDF TestMode" : "");
    const csvResultData = [
        header,
        ...domain.map((x, i) => {
            const row = [
                x,
                fittedCdfMin[i],
                fittedCdfMax[i],
                fittedPdfMin[i],
                fittedPdfMax[i],
                predictedCdf[i],
                predictedPdf[i],
                samplingCdf[i],
                samplingPdf[i],
            ];
            if (testMode) {
                row.push(testModeCdf[i], testModePdf[i]);
            }
            return row.join(",");
        })
    ].join("\n");

    // Create and download ZIP
    const zip = new JSZip();
    zip.file("data.csv", csvData);
    zip.file("result_info.csv", csvResultInfo);
    zip.file("frequency_histogram.csv", csvFreqHist);
    zip.file("estimated_cdf.csv", csvEstimatedCdf);
    zip.file("result_data.csv", csvResultData);

    const content = await zip.generateAsync({ type: "blob" });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(content);
    const timestamp = new Date().toISOString().replace(/[-:.]/g, "_");
    link.download = `quality_room_${timestamp}.zip`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
}
