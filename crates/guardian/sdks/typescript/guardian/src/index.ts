/**
 * Guardian TypeScript SDK - Type definitions and API client
 * Use with guardian CLI or Rust library via FFI
 */

export interface GuardianConfig {
  dbPath?: string;
  slopThreshold?: number;
  loopThreshold?: number;
}

export interface SlopIssue {
  line: number;
  patternId: string;
  patternName: string;
  severity: 'Critical' | 'High' | 'Medium' | 'Low' | 'Info';
  suggestion: string;
  category: 'PerformanceIssue' | 'MemorySafety' | 'IdiomaticRust' | 'ErrorHandling' | 'CodeStyle' | 'DesignPattern';
  code: string;
}

export interface YAGNIReport {
  unusedFunctions: number;
  unusedVariables: number;
  unusedImports: number;
  overEngineeringScore: number;
}

export interface OverEngineeringFinding {
  line: number;
  kind: 'Delete' | 'Stdlib' | 'Native' | 'Yagni' | 'Shrink';
  description: string;
  suggestion: string;
  deletableLines: number;
}

export interface OverEngineeringReport {
  findings: OverEngineeringFinding[];
  netDeletableLines: number;
  score: number;
}

export interface PonytailMark {
  line: number;
  text: string;
  shortcut: string;
  upgradePath?: string;
}

export interface PonytailCommentReport {
  marks: PonytailMark[];
  totalShortcuts: number;
}

export interface UncheckedFunction {
  name: string;
  line: number;
  hasBranches: boolean;
  hasLoops: boolean;
}

export interface MinimalCheckReport {
  unchecked: UncheckedFunction[];
  totalNontrivial: number;
  coveragePct: number;
}

export interface FileAnalysis {
  file: string;
  qualityScore: number;
  grade: string;
  slopIssues: SlopIssue[];
  yagniReport: YAGNIReport;
  overEngineering: OverEngineeringReport;
  ponytailMarks: PonytailCommentReport;
  minimalChecks: MinimalCheckReport;
}

export interface CommandFinding {
  kind: 'CommandLoop' | 'Destructive' | 'CdOscillation' | 'RepeatedFailure';
  index: number;
  severity: number;
  message: string;
}

export interface CommandLogReport {
  scanned: number;
  findings: CommandFinding[];
  commandFrequency: Record<string, number>;
}

/**
 * Guardian client - analyzes code files
 * Requires guardian binary in PATH or provide custom path
 */
export class Guardian {
  private config: GuardianConfig;
  private binaryPath: string;

  constructor(config: GuardianConfig = {}, binaryPath = 'guardian') {
    this.config = {
      slopThreshold: 10,
      loopThreshold: 3,
      ...config,
    };
    this.binaryPath = binaryPath;
  }

  /**
   * Analyze a single file using guardian CLI
   */
  async analyzeFile(filePath: string): Promise<FileAnalysis> {
    const { stdout } = await this.runGuardian(['--json', filePath]);
    return JSON.parse(stdout);
  }

  /**
   * Analyze multiple files
   */
  async analyzeFiles(filePaths: string[]): Promise<FileAnalysis[]> {
    return Promise.all(filePaths.map(f => this.analyzeFile(f)));
  }

  /**
   * Import command log and analyze patterns
   */
  async importAndAnalyzeCommands(logPath: string): Promise<CommandLogReport> {
    const { stdout } = await this.runGuardian(['import-log', logPath, '--json']);
    return JSON.parse(stdout);
  }

  /**
   * Show gain scoreboard
   */
  async showGain(): Promise<string> {
    const { stdout } = await this.runGuardian(['gain']);
    return stdout;
  }

  private async runGuardian(args: string[]): Promise<{ stdout: string; stderr: string }> {
    const fullArgs = ['--json', ...args];
    // In real implementation, use child_process.spawn
    // For now, return mock - actual FFI would use @napi-rs or similar
    throw new Error('Use guardian CLI directly or Rust FFI for full functionality');
  }
}

/**
 * Quick analysis without class - runs guardian CLI
 */
export async function analyze(filePath: string, options: GuardianConfig = {}): Promise<FileAnalysis> {
  const g = new Guardian(options);
  return g.analyzeFile(filePath);
}

/**
 * Default export for convenience
 */
export default Guardian;