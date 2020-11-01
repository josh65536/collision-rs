use cgmath::prelude::*;
use cgmath::{BaseFloat, Point3, Vector3};

use crate::prelude::*;
use crate::primitive::util::cylinder_ray_quadratic_solve;
use crate::volume::Sphere;
use crate::{Aabb3, Ray3};

/// Cylinder primitive
/// Cylinder body is aligned with the Y axis, with local origin in the center of the cylinders.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Cylinder<S> {
    half_height: S,
    radius: S,
}

impl<S> Cylinder<S>
where
    S: BaseFloat,
{
    /// Create a new cylinder
    pub fn new(half_height: S, radius: S) -> Self {
        Self {
            half_height,
            radius,
        }
    }

    /// Get radius
    pub fn radius(&self) -> S {
        self.radius
    }

    /// Get height
    pub fn height(&self) -> S {
        self.half_height + self.half_height
    }
}

impl<S> Primitive for Cylinder<S>
where
    S: BaseFloat,
{
    type Point = Point3<S>;

    fn support_point<T>(&self, direction: &Vector3<S>, transform: &T) -> Point3<S>
    where
        T: Transform<Point3<S>>,
    {
        let direction = transform.inverse_transform_vector(*direction).unwrap();

        let mut result = direction;
        let negative = result.y.is_sign_negative();

        result.y = S::zero();
        if result.magnitude2().is_zero() {
            result = Zero::zero();
        } else {
            result = result.normalize();
            if result.is_zero() {
                result = Zero::zero(); // cancel out any inconsistencies
            } else {
                result *= self.radius;
            }
        }
        if negative {
            result.y = -self.half_height;
        } else {
            result.y = self.half_height;
        }
        transform.transform_point(Point3::from_vec(result))
    }

    fn closest_valid_normal_local(
        &self,
        normal: &<Self::Point as EuclideanSpace>::Diff,
    ) -> <Self::Point as EuclideanSpace>::Diff {
        let flat = <Self::Point as EuclideanSpace>::Diff::new(normal.x, Zero::zero(), normal.z);

        if normal.y.abs() > flat.magnitude() {
            Vector3::new(Zero::zero(), normal.y.signum(), Zero::zero())
        } else {
            flat.normalize()
        }
    }
}

impl<S> ComputeBound<Aabb3<S>> for Cylinder<S>
where
    S: BaseFloat,
{
    fn compute_bound(&self) -> Aabb3<S> {
        Aabb3::new(
            Point3::new(-self.radius, -self.half_height, -self.radius),
            Point3::new(self.radius, self.half_height, self.radius),
        )
    }
}

impl<S> ComputeBound<Sphere<S>> for Cylinder<S>
where
    S: BaseFloat,
{
    fn compute_bound(&self) -> Sphere<S> {
        Sphere {
            center: Point3::origin(),
            radius: ((self.radius * self.radius) + (self.half_height * self.half_height)).sqrt(),
        }
    }
}

impl<S> Discrete<Ray3<S>> for Cylinder<S>
where
    S: BaseFloat,
{
    fn intersects(&self, r: &Ray3<S>) -> bool {
        if r.direction.x.is_zero() && r.direction.z.is_zero() {
            if r.direction.y.is_zero() {
                return false;
            }

            return (r.origin.y >= -self.half_height && r.direction.y <= S::zero())
                || (r.origin.y <= self.half_height && r.direction.y >= S::zero());
        }

        let (t1, t2) = match cylinder_ray_quadratic_solve(r, self.radius) {
            None => return false,
            Some(t) => t,
        };

        if t1 < S::zero() && t2 < S::zero() {
            return false;
        }

        let t = if t1 < S::zero() {
            t2
        } else if t2 < S::zero() {
            t1
        } else {
            t1.min(t2)
        };

        let pc = r.origin + r.direction * t;
        if pc.y <= self.half_height && pc.y >= -self.half_height {
            return true;
        }

        let n = -Vector3::unit_y();
        let tp = -(self.half_height + r.origin.dot(n)) / r.direction.dot(n);
        if tp >= S::zero() {
            let p = r.origin + r.direction * tp;
            if p.x * p.x + p.z * p.z < self.radius * self.radius {
                return true;
            }
        }

        let n = Vector3::unit_y();
        let tb = -(-self.half_height + r.origin.dot(n)) / r.direction.dot(n);
        if tb >= S::zero() {
            let p = r.origin + r.direction * tb;
            if p.x * p.x + p.z * p.z < self.radius * self.radius {
                return true;
            }
        }

        false
    }
}

impl<S> Continuous<Ray3<S>> for Cylinder<S>
where
    S: BaseFloat,
{
    type Result = Point3<S>;

    fn intersection(&self, r: &Ray3<S>) -> Option<Point3<S>> {
        self.intersection_normal(r).map(|(p, _)| p)
    }
}

impl<S> ContinuousNormal<Ray3<S>> for Cylinder<S>
where
    S: BaseFloat,
{
    type Point = Point3<S>;

    fn intersection_normal(
        &self,
        r: &Ray3<S>,
    ) -> Option<(Self::Point, <Self::Point as EuclideanSpace>::Diff)> {
        use cgmath::{vec2, vec3};

        if r.direction.x.is_zero() && r.direction.z.is_zero() {
            if r.direction.y.is_zero() || vec2(r.origin.x, r.origin.z).magnitude() > self.radius {
                return None;
            }

            if r.origin.y >= self.half_height && r.direction.y < S::zero() {
                return Some((
                    Point3::new(r.origin.x, self.half_height, r.origin.z),
                    Vector3::unit_y(),
                ));
            }
            if r.origin.y >= -self.half_height && r.direction.y < S::zero() {
                return Some((
                    Point3::new(r.origin.x, -self.half_height, r.origin.z),
                    -Vector3::unit_y(),
                ));
            }
            if r.origin.y <= -self.half_height && r.direction.y > S::zero() {
                return Some((
                    Point3::new(r.origin.x, -self.half_height, r.origin.z),
                    -Vector3::unit_y(),
                ));
            }
            if r.origin.y <= self.half_height && r.direction.y > S::zero() {
                return Some((
                    Point3::new(r.origin.x, self.half_height, r.origin.z),
                    Vector3::unit_y(),
                ));
            }

            return None;
        }

        let (t1, t2) = match cylinder_ray_quadratic_solve(r, self.radius) {
            None => return None,
            Some(t) => t,
        };

        if t1 < S::zero() && t2 < S::zero() {
            return None;
        }

        let mut t = if t1 < S::zero() {
            t2
        } else if t2 < S::zero() {
            t1
        } else {
            t1.min(t2)
        };

        // to avoid rounding error at cap collisions
        let mut hit_cap = false;
        let mut cap_y = S::zero();

        let mut normal = Vector3::zero();

        let n = -Vector3::unit_y();
        let tp = -(self.half_height + r.origin.dot(n)) / r.direction.dot(n);
        if tp >= S::zero() && tp < t {
            let p = r.origin + r.direction * tp;
            if p.x * p.x + p.z * p.z < self.radius * self.radius {
                t = tp;
                hit_cap = true;
                cap_y = self.half_height;
                normal = Vector3::unit_y();
            }
        }

        let n = Vector3::unit_y();
        let tb = -(-self.half_height + r.origin.dot(n)) / r.direction.dot(n);
        if tb >= S::zero() && tb < t {
            let p = r.origin + r.direction * tb;
            if p.x * p.x + p.z * p.z < self.radius * self.radius {
                t = tb;
                hit_cap = true;
                cap_y = -self.half_height;
                normal = -Vector3::unit_y();
            }
        }

        let mut pc = r.origin + r.direction * t;
        if hit_cap {
            pc.y = cap_y;
        } else {
            normal = vec3(pc.x, S::zero(), pc.z).normalize();
        }

        if (pc.y > self.half_height) || (pc.y < -self.half_height) {
            None
        } else {
            Some((pc, normal))
        }
    }
}

#[cfg(test)]
mod tests {
    use std;

    use approx::assert_ulps_eq;
    use cgmath::{vec3, Decomposed, Quaternion, Rad, Vector3};

    use super::*;

    #[test]
    fn test_cylinder_aabb() {
        let cylinder = Cylinder::new(2., 1.);
        assert_eq!(
            Aabb3::new(Point3::new(-1., -2., -1.), Point3::new(1., 2., 1.)),
            cylinder.compute_bound()
        );
    }

    #[test]
    fn test_cylinder_closest_valid_normal() {
        let cylinder = Cylinder::new(1., 1.);

        assert_eq!(
            vec3(0., 1., 0.),
            cylinder.closest_valid_normal_local(&vec3(-2.0 / 7.0, 6.0 / 7.0, 3.0 / 7.0))
        );
        assert_eq!(
            vec3(0., -1., 0.),
            cylinder.closest_valid_normal_local(&vec3(-2.0 / 7.0, -6.0 / 7.0, 3.0 / 7.0))
        );
        assert_eq!(
            vec3(-1., 0., 0.),
            cylinder.closest_valid_normal_local(&vec3(-0.8, 0.6, 0.0))
        );
    }

    #[test]
    fn test_cylinder_support_1() {
        let cylinder = Cylinder::new(2., 1.);
        let direction = Vector3::new(1., 0., 0.);
        let transform = transform(0., 0., 0., 0.);
        let point = cylinder.support_point(&direction, &transform);
        assert_ulps_eq!(Point3::new(1., 2., 0.), point);
    }

    #[test]
    fn test_cylinder_support_2() {
        let cylinder = Cylinder::new(2., 1.);
        let direction = Vector3::new(0.5, -1., 0.).normalize();
        let transform = transform(0., 0., 0., 0.);
        let point = cylinder.support_point(&direction, &transform);
        assert_ulps_eq!(Point3::new(1., -2., 0.), point);
    }

    #[test]
    fn test_cylinder_support_3() {
        let cylinder = Cylinder::new(2., 1.);
        let direction = Vector3::new(0., 1., 0.);
        let transform = transform(0., 0., 0., 0.);
        let point = cylinder.support_point(&direction, &transform);
        assert_ulps_eq!(Point3::new(0., 2., 0.), point);
    }

    #[test]
    fn test_cylinder_support_4() {
        let cylinder = Cylinder::new(2., 1.);
        let direction = Vector3::new(1., 0., 0.);
        let transform = transform(10., 0., 0., 0.);
        let point = cylinder.support_point(&direction, &transform);
        assert_ulps_eq!(Point3::new(11., 2., 0.), point);
    }

    #[test]
    fn test_cylinder_support_5() {
        let cylinder = Cylinder::new(2., 1.);
        let direction = Vector3::new(1., 0., 0.);
        let transform = transform(0., 0., 0., std::f32::consts::PI);
        let point = cylinder.support_point(&direction, &transform);
        assert_ulps_eq!(Point3::new(1., -2., 0.), point);
    }

    #[test]
    fn test_discrete_1() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(Point3::new(-3., 0., 0.), Vector3::new(1., 0., 0.));
        assert!(cylinder.intersects(&ray));
    }

    #[test]
    fn test_discrete_2() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(Point3::new(-3., 0., 0.), Vector3::new(-1., 0., 0.));
        assert!(!cylinder.intersects(&ray));
    }

    #[test]
    fn test_discrete_3() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0., 3., 0.),
            Vector3::new(0.1, -1., 0.1).normalize(),
        );
        assert!(cylinder.intersects(&ray));
    }

    #[test]
    fn test_discrete_4() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0., 3., 0.),
            Vector3::new(0.1, 1., 0.1).normalize(),
        );
        assert!(!cylinder.intersects(&ray));
    }

    #[test]
    fn test_continuous_1() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(Point3::new(-3., 0., 0.), Vector3::new(1., 0., 0.));
        assert_eq!(Some(Point3::new(-1., 0., 0.)), cylinder.intersection(&ray));
    }

    #[test]
    fn test_continuous_normal_1a() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(Point3::new(-3., 0., 0.), Vector3::new(1., 0., 0.));
        assert_eq!(
            Some((Point3::new(-1., 0., 0.), vec3(-1., 0., 0.))),
            cylinder.intersection_normal(&ray)
        );
    }

    #[test]
    fn test_continuous_normal_1b() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(Point3::new(-3., 0., 0.5), Vector3::new(4., 1., 0.));
        if let Some((p, n)) = cylinder.intersection_normal(&ray) {
            assert_ulps_eq!(
                Point3::new(-0.75f64.sqrt(), (-0.75f64.sqrt() + 3.) / 4., 0.5),
                p
            );
            assert_ulps_eq!(vec3(-0.75f64.sqrt(), 0., 0.5), n);
        } else {
            panic!("Intersection should exist");
        }
    }

    #[test]
    fn test_continuous_2() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(Point3::new(-3., 0., 0.), Vector3::new(-1., 0., 0.));
        assert_eq!(None, cylinder.intersection(&ray));
    }

    #[test]
    fn test_continuous_3() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0., 3., 0.),
            Vector3::new(0.1, -1., 0.1).normalize(),
        );
        assert_eq!(Some(Point3::new(0.1, 2., 0.1)), cylinder.intersection(&ray));
    }

    #[test]
    fn test_continuous_normal_3() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0., 3., 0.),
            Vector3::new(0.1, -1., 0.1).normalize(),
        );
        assert_eq!(
            Some((Point3::new(0.1, 2., 0.1), vec3(0., 1., 0.))),
            cylinder.intersection_normal(&ray)
        );
    }

    #[test]
    fn test_continuous_4() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0., 3., 0.),
            Vector3::new(0.1, 1., 0.1).normalize(),
        );
        assert_eq!(None, cylinder.intersection(&ray));
    }

    #[test]
    fn test_continuous_5() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0., 3., 0.),
            Vector3::new(0., -1., 0.).normalize(),
        );
        assert_eq!(Some(Point3::new(0., 2., 0.)), cylinder.intersection(&ray));
    }

    #[test]
    fn test_continuous_normal_5_up() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0., -3., 0.),
            Vector3::new(0., 1., 0.).normalize(),
        );
        assert_eq!(
            Some((Point3::new(0., -2., 0.), vec3(0., -1., 0.))),
            cylinder.intersection_normal(&ray)
        );
    }

    #[test]
    fn test_continuous_normal_5_down() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0., 3., 0.),
            Vector3::new(0., -1., 0.).normalize(),
        );
        assert_eq!(
            Some((Point3::new(0., 2., 0.), vec3(0., 1., 0.))),
            cylinder.intersection_normal(&ray)
        );
    }

    #[test]
    fn test_continuous_vertical_off_center_in() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0.5, 3., -0.25),
            Vector3::new(0., -1., 0.).normalize(),
        );
        assert_eq!(
            Some(Point3::new(0.5, 2., -0.25)),
            cylinder.intersection(&ray)
        );
    }

    #[test]
    fn test_continuous_normal_vertical_off_center_in() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0.5, 3., -0.25),
            Vector3::new(0., -1., 0.).normalize(),
        );
        assert_eq!(
            Some((Point3::new(0.5, 2., -0.25), vec3(0., 1., 0.))),
            cylinder.intersection_normal(&ray)
        );
    }

    #[test]
    fn test_continuous_vertical_off_center_out() {
        let cylinder = Cylinder::new(2., 1.);
        let ray = Ray3::new(
            Point3::new(0.5, 3., -1.),
            Vector3::new(0., -1., 0.).normalize(),
        );
        assert_eq!(None, cylinder.intersection(&ray));
    }

    // util
    fn transform(dx: f32, dy: f32, dz: f32, rot: f32) -> Decomposed<Vector3<f32>, Quaternion<f32>> {
        Decomposed {
            scale: 1.,
            rot: Quaternion::from_angle_z(Rad(rot)),
            disp: Vector3::new(dx, dy, dz),
        }
    }
}
