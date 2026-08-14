import {Composition} from "remotion";
import {Benchmark, benchmarkSchema, type BenchmarkProps} from "./video";

const defaultProps: BenchmarkProps = {caseId: "mixed-2d-3d", width: 1920, height: 1080, durationSeconds: 15};

export const Root = () => (
  <Composition
    id="CineKernelBenchmark"
    component={Benchmark}
    fps={30}
    width={1920}
    height={1080}
    durationInFrames={450}
    defaultProps={defaultProps}
    schema={benchmarkSchema}
    calculateMetadata={({props}) => ({width: props.width, height: props.height, durationInFrames: Math.round(props.durationSeconds * 30)})}
  />
);
