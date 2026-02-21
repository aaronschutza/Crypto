use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{AbstractField, PrimeField64};
use p3_goldilocks::Goldilocks;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

/// An Octonion represented by 8 elements in a field.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Octonion<F>(pub [F; 8]);

impl<F: AbstractField> Octonion<F> {
    /// Non-associative multiplication over the Fano Plane.
    /// This is the core "Serial bottleneck" of the VDF.
    pub fn mul(a: Self, b: Self) -> Self {
        let a = &a.0;
        let b = &b.0;
        let mut r = core::array::from_fn(|_| F::zero());

        r[0] = a[0].clone()*b[0].clone() - a[1].clone()*b[1].clone() - a[2].clone()*b[2].clone() - a[3].clone()*b[3].clone() - a[4].clone()*b[4].clone() - a[5].clone()*b[5].clone() - a[6].clone()*b[6].clone() - a[7].clone()*b[7].clone();
        r[1] = a[0].clone()*b[1].clone() + a[1].clone()*b[0].clone() + a[2].clone()*b[3].clone() - a[3].clone()*b[2].clone() + a[4].clone()*b[5].clone() - a[5].clone()*b[4].clone() - a[6].clone()*b[7].clone() + a[7].clone()*b[6].clone();
        r[2] = a[0].clone()*b[2].clone() - a[1].clone()*b[3].clone() + a[2].clone()*b[0].clone() + a[3].clone()*b[1].clone() + a[4].clone()*b[6].clone() + a[5].clone()*b[7].clone() - a[6].clone()*b[4].clone() - a[7].clone()*b[5].clone();
        r[3] = a[0].clone()*b[3].clone() + a[1].clone()*b[2].clone() - a[2].clone()*b[1].clone() + a[3].clone()*b[0].clone() + a[4].clone()*b[7].clone() - a[5].clone()*b[6].clone() + a[6].clone()*b[5].clone() - a[7].clone()*b[4].clone();
        r[4] = a[0].clone()*b[4].clone() - a[1].clone()*b[5].clone() - a[2].clone()*b[6].clone() - a[3].clone()*b[7].clone() + a[4].clone()*b[0].clone() + a[5].clone()*b[1].clone() + a[6].clone()*b[2].clone() + a[7].clone()*b[3].clone();
        r[5] = a[0].clone()*b[5].clone() + a[1].clone()*b[4].clone() - a[2].clone()*b[7].clone() + a[3].clone()*b[6].clone() - a[4].clone()*b[1].clone() + a[5].clone()*b[0].clone() - a[6].clone()*b[3].clone() + a[7].clone()*b[2].clone();
        r[6] = a[0].clone()*b[6].clone() + a[1].clone()*b[7].clone() + a[2].clone()*b[4].clone() - a[3].clone()*b[5].clone() - a[4].clone()*b[2].clone() + a[5].clone()*b[3].clone() + a[6].clone()*b[0].clone() - a[7].clone()*b[1].clone();
        r[7] = a[0].clone()*b[7].clone() - a[1].clone()*b[6].clone() + a[2].clone()*b[5].clone() + a[3].clone()*b[4].clone() - a[4].clone()*b[3].clone() - a[5].clone()*b[2].clone() + a[6].clone()*b[1].clone() + a[7].clone()*b[0].clone();

        Octonion(r)
    }

    pub fn add(a: Self, b: Self) -> Self {
        let mut r = core::array::from_fn(|_| F::zero());
        for i in 0..8 { r[i] = a.0[i].clone() + b.0[i].clone(); }
        Octonion(r)
    }

    pub fn sub(a: Self, b: Self) -> Self {
        let mut r = core::array::from_fn(|_| F::zero());
        for i in 0..8 { r[i] = a.0[i].clone() - b.0[i].clone(); }
        Octonion(r)
    }

    /// Associator: [A, B, D] = (AB)D - A(BD). 
    pub fn associator(a: Self, b: Self, d: Self) -> Self {
        let ab_d = Self::mul(Self::mul(a.clone(), b.clone()), d.clone());
        let a_bd = Self::mul(a, Self::mul(b, d));
        Self::sub(ab_d, a_bd)
    }
}

pub struct OctoStarkAir {
    pub c: Octonion<Goldilocks>,
    pub seed: Octonion<Goldilocks>,
    pub result: Octonion<Goldilocks>,
}

impl<F> BaseAir<F> for OctoStarkAir {
    fn width(&self) -> usize { 8 }
}

impl<AB: AirBuilder> Air<AB> for OctoStarkAir 
where AB::F: PrimeField64 
{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let next = main.row_slice(1);

        // --- 1. BOUNDARY CONSTRAINTS ---
        // We use as_canonical_u64() to bridge Goldilocks -> AB::F
        for i in 0..8 {
            let s_val = AB::F::from_canonical_u64(self.seed.0[i].as_canonical_u64());
            let r_val = AB::F::from_canonical_u64(self.result.0[i].as_canonical_u64());
            
            builder.when_first_row().assert_eq(local[i], s_val);
            builder.when_last_row().assert_eq(local[i], r_val);
        }

        // --- 2. TRANSITION CONSTRAINTS ---
        // Map local row to an Octonion of symbolic expressions
        let z_local = Octonion(core::array::from_fn(|i| local[i].into()));
        
        // Convert constant C into the builder's field context
        let c_expr = Octonion(core::array::from_fn(|i| {
            AB::Expr::from(AB::F::from_canonical_u64(self.c.0[i].as_canonical_u64()))
        }));

        // Algebraic Hash Injection H(Zn) = Zn^7
        let h_z_vals = core::array::from_fn(|i| {
            let x = z_local.0[i].clone();
            let x2 = x.clone() * x.clone();
            let x4 = x2.clone() * x2.clone();
            x4 * x2 * x 
        });
        let h_z = Octonion(h_z_vals);

        // Transition: Zn+1 = Zn^2 + C + [Zn, C, H(Zn)]
        let z_sq = Octonion::mul(z_local.clone(), z_local.clone());
        let assoc = Octonion::associator(z_local, c_expr.clone(), h_z);
        
        let expected_next = Octonion::add(Octonion::add(z_sq, c_expr), assoc);

        for i in 0..8 {
            builder.when_transition().assert_eq(next[i], expected_next.0[i].clone());
        }
    }
}

/// Sequential VDF implementation for the Prover (Goldilocks native).
pub fn run_vdf_grind(seed: Octonion<Goldilocks>, c: Octonion<Goldilocks>, t: usize) -> Vec<Octonion<Goldilocks>> {
    let mut history = Vec::with_capacity(t);
    let mut current = seed;
    for _ in 0..t {
        history.push(current);
        
        // H(Zn) = x^7 injection
        let h_z_vals = core::array::from_fn(|i| {
            let x = current.0[i];
            let x2 = x * x;
            let x4 = x2 * x2;
            x4 * x2 * x
        });
        let h_z = Octonion(h_z_vals);
        
        let z_sq = Octonion::mul(current, current);
        let assoc = Octonion::associator(current, c, h_z);
        current = Octonion::add(Octonion::add(z_sq, c), assoc);
    }
    history
}


pub fn test_octostark_vdf_trace() {
    let seed = Octonion([Goldilocks::from_canonical_u64(1); 8]);
    let c = Octonion([Goldilocks::from_canonical_u64(42); 8]);
    let t = 128; 

    let trace_history = run_vdf_grind(seed, c, t);
    let result = *trace_history.last().unwrap();

    assert_ne!(seed, result, "VDF must result in a state change");
    
    let mut trace_flat = Vec::new();
    for step in trace_history { trace_flat.extend_from_slice(&step.0); }
    let trace_matrix = RowMajorMatrix::new(trace_flat, 8);

    println!("VDF Computed Successfully.");
    println!("Trace Height: {}", trace_matrix.height());
    println!("Final State e0: {:?}", result.0[0]);
}
